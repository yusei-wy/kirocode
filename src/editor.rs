use crate::error::{Error, Result};
use crate::input::StdinRawMode;
use crate::screen::Screen;

use std::io::Write;

#[derive(PartialEq)]
pub enum Sequence {
    AllowLeft,
    AllowRight,
    AllowUp,
    AllowDown,
    Del,
    Home,
    End,
    PageUp,
    PageDown,
    Key(u8),
}

pub struct Editor<W: Write> {
    screen: Screen<W>,
    input: StdinRawMode,
}

impl<W> Editor<W>
where
    W: Write,
{
    pub fn open(output: W) -> Result<Self> {
        let mut input = StdinRawMode::new()?;
        let mut screen = Screen::new(None, &mut input, output)?;
        screen.refresh()?;

        Ok(Self { screen, input })
    }

    pub fn edit(&mut self) -> Result<()> {
        self.screen.refresh()?;

        loop {
            self.screen.refresh()?;
            let ok = self.process_keypress()?;
            if !ok {
                self.screen.clear()?;
                break;
            }
        }

        Ok(())
    }

    fn process_keypress(&mut self) -> Result<bool> {
        let seq = self.read_key()?;

        match seq {
            Sequence::Key(b) => {
                if b == ctrl_key('q') {
                    return Ok(false);
                }
            }

            Sequence::Home => {
                self.screen.set_cx(0);
            }

            Sequence::End => {
                self.screen.set_cx(self.screen.cols() - 1);
            }

            Sequence::PageUp | Sequence::PageDown => {
                let mut times = self.screen.rows();
                loop {
                    times -= 1;
                    if times == 0 {
                        break;
                    }
                    if seq == Sequence::PageUp {
                        self.screen.move_cursor(Sequence::AllowUp);
                    } else {
                        self.screen.move_cursor(Sequence::AllowDown);
                    }
                }
            }

            Sequence::AllowUp
            | Sequence::AllowDown
            | Sequence::AllowRight
            | Sequence::AllowLeft => self.screen.move_cursor(seq),

            Sequence::Del => {}
        }

        Ok(true)
    }

    fn read_key(&mut self) -> Result<Sequence> {
        let ob = self.input.read_byte()?;
        let b = ob.ok_or(Error::InputReadByteError)?;
        if b != b'\x1b' {
            return Ok(Sequence::Key(b));
        }

        let mut seq: Vec<u8> = Vec::with_capacity(3);
        seq.push(self.input.read_byte()?.ok_or(Error::InputReadByteError)?);
        seq.push(self.input.read_byte()?.ok_or(Error::InputReadByteError)?);

        if seq[0] == b'[' {
            if b'0' <= seq[1] && seq[1] <= b'9' {
                seq.push(self.input.read_byte()?.ok_or(Error::InputReadByteError)?);
                if seq[2] == b'~' {
                    match seq[1] {
                        b'1' | b'7' => return Ok(Sequence::Home),
                        b'4' | b'8' => return Ok(Sequence::End),
                        b'3' => return Ok(Sequence::Del),
                        b'5' => return Ok(Sequence::PageUp),
                        b'6' => return Ok(Sequence::PageDown),
                        _ => {}
                    }
                }
            } else {
                match seq[1] {
                    b'A' => return Ok(Sequence::AllowUp),
                    b'B' => return Ok(Sequence::AllowDown),
                    b'C' => return Ok(Sequence::AllowRight),
                    b'D' => return Ok(Sequence::AllowLeft),
                    b'H' => return Ok(Sequence::Home),
                    b'F' => return Ok(Sequence::End),
                    _ => {}
                }
            }
        } else if seq[0] == b'O' {
            match seq[1] {
                b'H' => return Ok(Sequence::Home),
                b'F' => return Ok(Sequence::End),
                _ => {}
            }
        }

        Ok(Sequence::Key(b'\x1b'))
    }
}

fn ctrl_key(c: char) -> u8 {
    c as u8 & 0x1f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_key() {
        assert_eq!(ctrl_key('q'), 17);
    }
}
