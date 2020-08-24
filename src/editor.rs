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

        if let Ok(lines) = read_lines(filepath) {
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
            InputSeq {
                key, ctrl: true, ..
            } => match key {
                Key(b'q') => return Ok(false),
                _ => {}
            },
            InputSeq { key, .. } => match key {
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

fn read_lines<P>(filepath: P) -> Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filepath)?;
    Ok(io::BufReader::new(file).lines())
}

mod tests {
    use super::*;

    use crate::error::Error;
    use crate::input::{DummyInputSequences, KeySeq};
    use KeySeq::*;

    #[test]
    fn test_read_lines() {
        let lines = read_lines("");
        if let Err(err) = lines {
            match err {
                Error::IoError(_) => {}
                _ => unreachable!(),
            }
        }

        if let Ok(lines) = read_lines("./test.txt") {
            let mut cnt = 0;
            for line in lines {
                cnt += 1;
                if let Ok(ip) = line {
                    assert_eq!(ip, "kirocode test file.");
                }
            }
            assert_eq!(cnt, 1);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_editor_new() {
        let i = DummyInputSequences(vec![]);
        let o: Vec<u8> = vec![];
        let e = Editor::new(i, o).unwrap();
        assert_eq!(e.buf_rows, 0);
        assert_eq!(e.rows.len(), 0);
    }

    #[test]
    fn test_editor_open() {
        let i = DummyInputSequences(vec![]);
        let o: Vec<u8> = vec![];
        let e = Editor::open("", i, o).unwrap();
        assert_eq!(e.buf_rows, 1);
        assert_eq!(e.rows.len(), 0);

        let i = DummyInputSequences(vec![]);
        let o: Vec<u8> = vec![];
        let e = Editor::open("./test.txt", i, o).unwrap();
        assert_eq!(e.buf_rows, 2);
        assert_eq!(e.rows.len(), 1);

        assert_eq!(e.rows[0].size, 19);
        assert_eq!(e.rows[0].buf, b"kirocode test file.");
    }

    #[test]
    fn test_append_row() {
        let i = DummyInputSequences(vec![]);
        let o: Vec<u8> = vec![];
        let mut e = Editor::new(i, o).unwrap();

        e.append_row(b"kirocode".to_vec(), 8);
        assert_eq!(e.rows.len(), 1);
        assert_eq!(e.rows[0].size, 8);
        assert_eq!(e.rows[0].buf, b"kirocode");
        assert_eq!(e.buf_rows, 1);
    }

    #[test]
    fn test_process_keypress() {
        let i = DummyInputSequences(vec![]);
        let o: Vec<u8> = vec![];
        let mut e = Editor::new(i, o).unwrap();

        let ret = e.process_keypress(InputSeq::new(Key(b'a')));
        assert_eq!(ret.unwrap(), true);

        // quit
        let ret = e.process_keypress(InputSeq::ctrl(Key(b'q')));
        assert_eq!(ret.unwrap(), false);
    }
}
