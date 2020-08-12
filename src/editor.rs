use crate::error::Result;
use crate::input::{InputSeq, KeySeq};
use crate::screen::Screen;

use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

pub struct Editor<I: Iterator<Item = Result<InputSeq>>, W: Write> {
    screen: Screen<W>,
    input: I,
    buf_rows: usize,
    rows: Vec<EditorRow>,
}

pub struct EditorRow {
    pub size: usize,
    pub buf: Vec<u8>,
}

impl<I, W> Editor<I, W>
where
    I: Iterator<Item = Result<InputSeq>>,
    W: Write,
{
    pub fn open<P: AsRef<Path>>(filepath: P, mut input: I, output: W) -> Result<Self> {
        let screen = Screen::new(None, &mut input, output)?;

        let mut editor = Self {
            screen,
            input,
            buf_rows: 1,
            rows: Vec::new(),
        };

        if let Ok(lines) = Self::read_lines(filepath) {
            for line in lines {
                if let Ok(ip) = line {
                    let buf = ip.into_bytes();
                    let mut size = buf.len();
                    loop {
                        if size > 0 && (buf[size - 1] == b'\n' || buf[size - 1] == b'\r') {
                            size -= 1;
                        }
                        break;
                    }

                    editor.append_row(buf, size);
                }
            }
        }

        Ok(editor)
    }

    pub fn new(mut input: I, output: W) -> Result<Self> {
        let screen = Screen::new(None, &mut input, output)?;

        let editor = Self {
            screen,
            input,
            buf_rows: 0,
            rows: Vec::new(),
        };

        Ok(editor)
    }

    fn read_lines<P>(filepath: P) -> Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filepath)?;
        Ok(io::BufReader::new(file).lines())
    }

    fn append_row(&mut self, buf: Vec<u8>, len: usize) {
        self.rows.push(EditorRow { size: len, buf });
        self.buf_rows += 1;
    }

    pub fn edit(&mut self) -> Result<()> {
        self.screen.refresh(self.buf_rows, &mut self.rows)?;

        loop {
            self.screen.refresh(self.buf_rows, &mut self.rows)?;
            if let Some(seq) = self.input.next() {
                let ok = self.process_keypress(seq?)?;
                if !ok {
                    self.screen.clear()?;
                    break;
                }
            } else {
                self.screen.clear()?;
                break;
            }
        }

        Ok(())
    }

    fn process_keypress(&mut self, seq: InputSeq) -> Result<bool> {
        use KeySeq::*;
        match seq {
            // FIXME: ctrl: true が処理できない
            InputSeq {
                key, ctrl: true, ..
            } => match key {
                Key(b'q') => return Ok(false),
                _ => {}
            },
            InputSeq { key, .. } => match key {
                // TODO: テスト用
                Key(b'q') => return Ok(false),

                Home => self.screen.set_cx(0),
                End => self.screen.set_cx(self.screen.cols() - 1),
                PageUp | PageDown => {
                    let mut times = self.screen.rows();
                    loop {
                        times -= 1;
                        if times == 0 {
                            break;
                        }
                        if key == PageUp {
                            self.screen.move_cursor(Up, self.buf_rows);
                        } else {
                            self.screen.move_cursor(Down, self.buf_rows);
                        }
                    }
                }
                Up | Down | Right | Left => self.screen.move_cursor(key, self.buf_rows),
                Del => {}
                _ => {}
            },
        }

        Ok(true)
    }
}
