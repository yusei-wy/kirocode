use crate::error::{Error, Result};
use crate::input::StdinRawMode;

use std::io::Write;
use std::str::FromStr;

pub struct Screen<W: Write> {
    pub rows: usize,
    pub cols: usize,
    output: W,
    buf: Vec<u8>,
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
        let buf = Vec::new();
        if let Some((w, h)) = size {
            return Ok(Self {
                rows: w,
                cols: h,
                output,
                buf,
            });
        }

        let (w, h) = get_window_size(input, &mut output)?;
        Ok(Self {
            rows: w,
            cols: h,
            output,
            buf,
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        self.output.write(b"\x1b[2J")?;
        self.output.write(b"\x1b[H")?;
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.clear()?;

        self.draw_rows(self.rows);

        self.append_buffers(b"\x1b[H");

        let b = &self.buf;
        self.output.write(b)?;
        Ok(())
    }

    pub fn draw_rows(&mut self, rows: usize) {
        for y in 0..rows {
            self.append_buffers(b"~");

            if y < self.rows - 1 {
                self.append_buffers(b"\r\n");
            }
        }
    }

    fn append_buffers(&mut self, buf: &[u8]) {
        let buf = buf.iter().map(|b| *b).collect::<Vec<u8>>();
        for b in buf {
            self.buf.push(b);
        }
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
