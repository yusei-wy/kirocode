use crate::error::Result;
use crate::input::{InputSeq, KeySeq};
use crate::screen::Screen;
use std::io::Write;

pub struct Editor<I: Iterator<Item = Result<InputSeq>>, W: Write> {
    input: I,
    screen: Screen<W>,
}

impl<I, W> Editor<I, W>
where
    I: Iterator<Item = Result<InputSeq>>,
    W: Write,
{
    pub fn new(
        mut input: I,
        output: W,
        window_size: Option<(usize, usize)>,
    ) -> Result<Editor<I, W>> {
        let screen = Screen::new(window_size, &mut input, output)?;

        Ok(Editor { input, screen })
    }

    // WHY: mut が変数の前に来る？
    pub fn open(
        mut input: I,
        output: W,
        window_size: Option<(usize, usize)>,
    ) -> Result<Editor<I, W>> {
        Self::new(input, output, window_size)
    }

    pub fn edit(&mut self) -> Result<()> {
        self.refresh_screen();
        self.screen.flush();

        loop {
            self.refresh_screen();
            if !self.process_keypress() {
                self.screen.render("\x1b[2J");
                self.screen.render("\x1b[H");
                break;
            }
        }

        Ok(())
    }

    fn refresh_screen(&mut self) {
        self.screen.render("\x1b[2J");
        self.screen.render("\x1b[H");

        self.draw_rows();

        self.screen.render("\x1b[H");
    }

    fn draw_rows(&mut self) {
        for _ in 0..self.screen.rows() {
            self.screen.render("~\r\n");
        }
    }

    fn process_keypress(&mut self) -> bool {
        let input_seq = self.read_key();

        if input_seq.ctrl && input_seq.key == KeySeq::Key(b'q') {
            return false;
        }

        true
    }

    fn read_key(&mut self) -> InputSeq {
        self.input.next().unwrap().unwrap()
    }
}
