use crate::error::{Error, Result};
use crate::input::StdinRawMode;
use crate::screen::Screen;

use std::io::{self, BufWriter, Write};

pub enum Sequence {
    AllowLeft,
    AllowRight,
    AllowUp,
    AllowDown,
    Key(u8),
}

pub fn edit() -> Result<()> {
    let mut input = StdinRawMode::new()?;
    let output = io::stdout();
    let output = BufWriter::new(output.lock());

    let mut screen = Screen::new(None, &mut input, output)?;
    screen.refresh()?;

    loop {
        screen.refresh()?;
        let ok = editor_process_keypress(&mut input, &mut screen)?;
        if !ok {
            screen.clear()?;
            break;
        }
    }

    Ok(())
}

fn editor_process_keypress<W>(input: &mut StdinRawMode, screen: &mut Screen<W>) -> Result<bool>
where
    W: Write,
{
    let seq = editor_read_key(input)?;

    match seq {
        Sequence::Key(b) => {
            if b == ctrl_key('q') {
                return Ok(false);
            }
        }
        Sequence::AllowUp | Sequence::AllowDown | Sequence::AllowRight | Sequence::AllowLeft => {
            screen.move_cursor(seq)
        }
    }

    Ok(true)
}

fn editor_read_key(input: &mut StdinRawMode) -> Result<Sequence> {
    let ob = input.read_byte()?;
    let b = ob.ok_or(Error::InputReadByteError)?;
    if b != b'\x1b' {
        return Ok(Sequence::Key(b));
    }

    let mut seq: Vec<u8> = Vec::with_capacity(3);
    seq.push(input.read_byte()?.ok_or(Error::InputReadByteError)?);
    seq.push(input.read_byte()?.ok_or(Error::InputReadByteError)?);

    if seq[0] != b'[' {
        return Ok(Sequence::Key(b'\x1b'));
    }

    match seq[1] {
        b'A' => return Ok(Sequence::AllowUp),
        b'B' => return Ok(Sequence::AllowDown),
        b'C' => return Ok(Sequence::AllowRight),
        b'D' => return Ok(Sequence::AllowLeft),
        _ => {}
    }

    Ok(Sequence::Key(b'\x1b'))
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
