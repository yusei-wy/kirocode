use crate::input::{InputSeq, KeySeq, StdinRawMode};
use crate::screen::Screen;
use std::io::{self, BufWriter, Write};

pub struct Editor<'a> {
    pub input: StdinRawMode,
    pub out: BufWriter<io::StdoutLock<'a>>,
    pub screen: Screen,
}

impl<'a> Editor<'a> {
    pub fn new(input: StdinRawMode, out: BufWriter<io::StdoutLock<'a>>) -> Editor {
        Editor {
            input,
            out,
            screen: Screen::new(None),
        }
    }

    pub fn edit(&mut self) {
        self.input.enable_raw_mode();
        self.refresh_screen();
        self.out.flush().unwrap();

        loop {
            self.refresh_screen();
            if !self.process_keypress() {
                write!(self.out, "\x1b[2J").unwrap();
                write!(self.out, "\x1b[H").unwrap();
                break;
            }
        }
    }

    fn refresh_screen(&mut self) {
        write!(self.out, "\x1b[2J").unwrap();
        write!(self.out, "\x1b[H").unwrap();

        self.draw_rows();

        write!(self.out, "\x1b[H").unwrap();
    }

    fn draw_rows(&mut self) {
        for _ in 0..self.screen.rows {
            write!(self.out, "~\r\n").unwrap();
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
        let b = self.input.read_byte().unwrap();
        self.input.decode(b.unwrap()).unwrap()
    }
}
