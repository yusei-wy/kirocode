use crate::editor::EditorRow;
use crate::error::{Error, Result};
use crate::input::{InputSeq, KeySeq};

use std::io::Write;

const VERSION: &str = "0.0.1";

pub struct Screen<W: Write> {
    cx: usize,
    cy: usize,
    rows: usize,
    cols: usize,
    row_off: usize,
    col_off: usize,
    output: W,
    buf: Vec<u8>,
}

impl<W> Screen<W>
where
    W: Write,
{
    pub fn new<I>(size: Option<(usize, usize)>, input: I, mut output: W) -> Result<Self>
    where
        I: Iterator<Item = Result<InputSeq>>,
    {
        let buf = Vec::new();
        if let Some((w, h)) = size {
            return Ok(Self {
                cx: 0,
                cy: 0,
                rows: h,
                cols: w,
                row_off: 0,
                col_off: 0,
                output,
                buf,
            });
        }

        let (w, h) = get_window_size(input, &mut output)?;
        Ok(Self {
            cx: 0,
            cy: 0,
            rows: h,
            cols: w,
            row_off: 0,
            col_off: 0,
            output,
            buf,
        })
    }

    // getter

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    // setter

    pub fn set_cx(&mut self, cx: usize) {
        self.cx = cx;
    }

    pub fn clear(&mut self) -> Result<()> {
        self.output.write(b"\x1b[2J")?;
        self.output.write(b"\x1b[H")?;
        Ok(())
    }

    pub fn refresh(&mut self, num_size: usize, rows: &mut Vec<EditorRow>) -> Result<()> {
        self.scroll();

        self.append_buffers(b"\x1b[?25l", 4);
        self.append_buffers(b"\x1b[H", 3);

        self.draw_rows(num_size, rows);

        let buf = format!("\x1b[{};{}H", (self.cy - self.row_off) + 1, self.cx + 1);
        self.append_buffers(buf.as_bytes(), buf.len());

        self.append_buffers(b"\x1b[?25h", 6);

        let b = &self.buf;
        self.output.write(b)?;
        self.output.flush()?; // 描画後は flush しないとカーソルの位置が上に戻らない
        self.free_buffers();

        Ok(())
    }

    fn draw_rows(&mut self, num_rows: usize, rows: &mut Vec<EditorRow>) {
        for y in 0..self.rows {
            let file_row = y + self.row_off;
            if file_row >= num_rows {
                if num_rows == 0 && y == self.rows / 3 {
                    let welcom = format!("KiroCode -- version {}", VERSION);
                    let welcom_len = if welcom.len() > self.cols {
                        self.cols
                    } else {
                        welcom.len()
                    };
                    let padding: i32 = (self.cols - welcom_len) as i32;
                    let mut padding = padding / 2;
                    if padding > 0 {
                        self.append_buffers(b"~", 1);
                        padding -= 1;
                    }
                    loop {
                        padding -= 1;
                        if padding > 0 {
                            self.append_buffers(b" ", 1);
                        } else {
                            break;
                        }
                    }
                    self.append_buffers(welcom.as_bytes(), welcom_len);
                } else {
                    self.append_buffers(b"~", 1);
                }
            } else {
                if let Some(row) = rows.get(file_row) {
                    if row.size > self.cols {
                        self.append_buffers(&row.buf, self.cols);
                    } else {
                        self.append_buffers(&row.buf, row.size);
                    }
                }
            }

            self.append_buffers(b"\x1b[K", 3);
            if y < self.rows - 1 {
                self.append_buffers(b"\r\n", 2);
            }
        }
    }

    fn append_buffers(&mut self, buf: &[u8], len: usize) {
        self.buf.extend(buf[..len].iter());
    }

    fn free_buffers(&mut self) {
        self.buf = Vec::new();
    }

    fn scroll(&mut self) {
        // カーソルが可視ウィンドウ上にあるなら、カーソル位置まで移動
        if self.cy < self.row_off {
            self.row_off = self.cy;
        }
        // カーソルが可視ウィンドウの下部を超えているなら、カーソルを画面の下部で固定
        if self.cy >= self.row_off + self.rows {
            self.row_off = self.cy - self.rows + 1;
        }
    }

    pub fn move_cursor(&mut self, key: KeySeq, buf_rows: usize) {
        use KeySeq::*;
        match key {
            Left => {
                if self.cx > 0 {
                    self.cx -= 1;
                }
            }
            Right => {
                if self.cx < self.cols {
                    self.cx += 1
                }
            }
            Up => {
                if self.cy > 0 {
                    self.cy -= 1;
                }
            }
            Down => {
                if self.cy < buf_rows {
                    self.cy += 1;
                }
            }
            _ => {}
        }
    }
}

fn get_window_size<I, W>(input: I, output: W) -> Result<(usize, usize)>
where
    I: Iterator<Item = Result<InputSeq>>,
    W: Write,
{
    if let Some(s) = term_size::dimensions() {
        return Ok(s);
    }

    get_cursor_pos(input, output)
}

fn get_cursor_pos<I, W>(input: I, mut output: W) -> Result<(usize, usize)>
where
    I: Iterator<Item = Result<InputSeq>>,
    W: Write,
{
    // カーソルを画面右下に移動してフォールバックとしてサイズを取得する
    output.write(b"\x1b[999C\x1b[999B\x1b[6n")?;
    output.flush()?;

    for seq in input {
        if let KeySeq::Cursor(w, h) = seq?.key {
            return Ok((w, h));
        }
    }

    Err(Error::UnknownWindowSize)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::error::Error;
    use crate::input::DummyInputSequences;

    use KeySeq::*;

    #[test]
    fn test_screen_new() {
        let input = DummyInputSequences(vec![]);
        let output: Vec<u8> = vec![];
        match Screen::new(None, input, output) {
            Ok(screen) => {
                assert!(screen.cols > 0);
                assert!(screen.rows > 0);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_screen_new_default_size() {
        let input = DummyInputSequences(vec![]);
        let output: Vec<u8> = vec![];
        match Screen::new(Some((50, 100)), input, output) {
            Ok(screen) => {
                assert_eq!(screen.cols, 50);
                assert_eq!(screen.rows, 100);
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn test_get_cursor_pos() {
        let input = DummyInputSequences(vec![]);
        let mut output: Vec<u8> = vec![];

        match get_cursor_pos(input, &mut output) {
            Err(Error::UnknownWindowSize) => {}
            _ => unreachable!(),
        }

        let input = DummyInputSequences(vec![InputSeq::new(Cursor(50, 100))]);
        let mut output: Vec<u8> = vec![];
        match get_cursor_pos(input, &mut output) {
            Ok((x, y)) => {
                assert_eq!(x, 50);
                assert_eq!(y, 100);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_clear() {
        let input = DummyInputSequences(vec![]);
        let mut output: Vec<u8> = vec![];
        let mut screen = Screen::new(Some((50, 100)), input, &mut output).unwrap();
        screen.clear().unwrap();
        assert_eq!(output, b"\x1b[2J\x1b[H");
    }

    #[test]
    fn test_append_buffers() {
        let input = DummyInputSequences(vec![]);
        let output: Vec<u8> = vec![];
        let mut screen = Screen::new(Some((50, 100)), input, output).unwrap();
        screen.append_buffers(b"abcde", 3);
        assert_eq!(screen.buf, vec![b'a', b'b', b'c']);
    }

    #[test]
    fn test_free_buffers() {
        let input = DummyInputSequences(vec![]);
        let output: Vec<u8> = vec![];
        let mut screen = Screen::new(Some((50, 100)), input, output).unwrap();
        screen.append_buffers(b"abcde", 3);
        assert_eq!(screen.buf, vec![b'a', b'b', b'c']);
        screen.free_buffers();
        assert_eq!(screen.buf, vec![]);
    }

    #[test]
    fn test_move_cursor() {
        // left
        let input = DummyInputSequences(vec![]);
        let output: Vec<u8> = vec![];
        let mut screen = Screen::new(Some((50, 100)), input, output).unwrap();
        screen.move_cursor(Left, 1);
        assert_eq!(screen.cx, 0);
        screen.move_cursor(Right, 1);
        assert_eq!(screen.cx, 1);

        // right
        let input = DummyInputSequences(vec![]);
        let output: Vec<u8> = vec![];
        let mut screen = Screen::new(Some((50, 100)), input, output).unwrap();
        for _ in 0..100 {
            screen.move_cursor(Right, 1);
        }
        assert_eq!(screen.cx, 50);

        // up
        let input = DummyInputSequences(vec![]);
        let output: Vec<u8> = vec![];
        let mut screen = Screen::new(Some((50, 100)), input, output).unwrap();
        screen.move_cursor(Up, 10);
        assert_eq!(screen.cx, 0);
        screen.move_cursor(Down, 10);
        assert_eq!(screen.cy, 1);

        // down
        let input = DummyInputSequences(vec![]);
        let output: Vec<u8> = vec![];
        let mut screen = Screen::new(Some((50, 100)), input, output).unwrap();
        for _ in 0..200 {
            screen.move_cursor(Down, 10);
        }
        assert_eq!(screen.cy, 10);
        for _ in 0..200 {
            screen.move_cursor(Down, 100);
        }
        assert_eq!(screen.cy, 100);
    }

    #[test]
    fn test_draw_rows_welcom_message() {
        let i = DummyInputSequences(vec![]);
        let o: Vec<u8> = vec![];
        let mut s = Screen::new(Some((50, 100)), i, o).unwrap();
        s.draw_rows(0, &mut vec![]);

        let mut buf: Vec<u8> = vec![];
        for _ in 0..33 {
            buf.extend(b"~\x1b[K\r\n");
        }
        buf.extend(b"~");
        // NOTE: 12かと思ったけどなぜか10. 理由がわかっていない
        for _ in 0..10 {
            buf.extend(b" ");
        }
        buf.extend(b"KiroCode -- version 0.0.1\x1b[K\r\n");
        for _ in 0..65 {
            buf.extend(b"~\x1b[K\r\n");
        }
        buf.extend(b"~\x1b[K");

        assert_eq!(
            String::from_utf8(s.buf).unwrap(),
            String::from_utf8(buf).unwrap(),
        );
    }
}
