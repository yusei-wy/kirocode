use crate::error::{Error, Result};
use crate::input::InputSeq;
use std::io::Write;

pub struct Screen<W: Write> {
    rows: usize,
    cols: usize,
    output: W,
}

impl<W: Write> Screen<W> {
    pub fn new<I>(size: Option<(usize, usize)>, input: I, mut output: W) -> Result<Self>
    where
        I: Iterator<Item = Result<InputSeq>>,
    {
        let (w, h) = if let Some(s) = size {
            s
        } else {
            get_window_size(input, &mut output)?
        };

        Ok(Screen {
            rows: w,
            cols: h,
            output,
        })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn flush(&mut self) {
        self.output.flush().unwrap();
    }

    pub fn render(&mut self, text: &str) {
        write!(self.output, "{}", text).unwrap();
    }
}

fn get_window_size<I, W>(_input: I, output: &mut W) -> Result<(usize, usize)>
where
    W: Write,
{
    if let Some(s) = term_size::dimensions_stdout() {
        return Ok(s);
    };

    write!(output, "\x1b[999C\x1b[999B").unwrap();
    output.flush().unwrap();

    Err(Error::UnknownWindowSize)
}
