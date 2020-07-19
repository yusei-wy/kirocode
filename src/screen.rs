use crate::error::{Error, Result};
use crate::input::StdinRawMode;

use std::io::Write;
use std::str::FromStr;

pub struct Screen<W: Write> {
    pub rows: usize,
    pub cols: usize,
    output: W,
}

impl<W> Screen<W>
where
    W: Write,
{
    pub fn new(
        size: Option<(usize, usize)>,
        input: &mut StdinRawMode,
        mut output: W,
    ) -> Result<Self> {
        if let Some((w, h)) = size {
            return Ok(Self {
                rows: w,
                cols: h,
                output,
            });
        }

        let (w, h) = get_window_size(input, &mut output)?;
        Ok(Self {
            rows: w,
            cols: h,
            output,
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        self.output.write(b"\x1b[2J")?;
        self.output.write(b"\x1b[H")?;
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.clear()?;

        self.draw_rows(self.rows)?;

        self.output.write(b"\x1b[H")?;
        self.output.flush()?;
        Ok(())
    }

    pub fn draw_rows(&mut self, rows: usize) -> Result<()> {
        for y in 0..rows {
            self.output.write(b"~")?;

            if y < self.rows - 1 {
                self.output.write(b"\r\n")?;
            }
        }
        Ok(())
    }
}

fn get_window_size<W>(input: &mut StdinRawMode, output: &mut W) -> Result<(usize, usize)>
where
    W: Write,
{
    if let Some(s) = term_size::dimensions() {
        return Ok(s);
    }

    output.write(b"\x1b[999C\x1b[999B")?;
    output.flush()?;

    let (w, h) = get_cursor_position(input, output)?;

    Ok((w, h))
}

fn get_cursor_position<W>(input: &mut StdinRawMode, output: &mut W) -> Result<(usize, usize)>
where
    W: Write,
{
    let mut buf: Vec<u8> = Vec::with_capacity(32);
    let mut i: usize = 0;

    output.write(b"\x1b[6n")?;
    output.flush()?;

    loop {
        if i >= buf.capacity() - 1 {
            break;
        }
        let ob = input.read_byte()?;
        if let Some(b) = ob {
            buf.push(b);
            if b == b'R' {
                break;
            }
        }
        i += 1;
    }
    buf[i] = b'\0';

    if buf[0] != b'\x1b' || buf[1] != b'[' {
        return Err(Error::InputNotFoundEscapeError);
    }
    let buf_str = buf[2..].iter().map(|&s| s as char).collect::<String>();
    let s = buf_str.split('\0').collect::<Vec<&str>>()[0]
        .split(';')
        .collect::<Vec<&str>>();
    if s.len() != 2 {
        return Err(Error::ScreenGetSizeError);
    }

    let w = usize::from_str(s[0])?;
    let h = usize::from_str(s[1])?;
    Ok((w, h))
}
