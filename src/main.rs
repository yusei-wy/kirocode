use kirocode::{Error, Result, Screen, StdinRawMode};

use std::io::{self, BufWriter, Write};

fn main() {
    if let Err(err) = edit() {
        die(err);
    }
}

fn die(err: Error) {
    print!("\x1b[2J");
    print!("\x1b[H");
    eprintln!("{}", err);
}

// TODO: editor 関係の処理を別ファイルに分離する

fn edit() -> Result<()> {
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
    let b = editor_read_key(input)?;

    if b == ctrl_key('q') {
        return Ok(false);
    }

    match b {
        b'w' | b's' | b'a' | b'd' => screen.move_cursor(b),
        _ => {}
    }

    Ok(true)
}

fn editor_read_key(input: &mut StdinRawMode) -> Result<u8> {
    let ob = input.read_byte()?;
    let b = ob.ok_or(Error::InputReadByteError)?;
    if b != b'\x1b' {
        return Ok(b);
    }

    let mut seq: Vec<u8> = Vec::with_capacity(3);
    seq.push(input.read_byte()?.ok_or(Error::InputReadByteError)?);
    seq.push(input.read_byte()?.ok_or(Error::InputReadByteError)?);

    if seq[0] != b'[' {
        return Ok(b'\x1b');
    }

    match seq[1] {
        b'A' => return Ok(b'w'),
        b'B' => return Ok(b's'),
        b'C' => return Ok(b'd'),
        b'D' => return Ok(b'a'),
        _ => {}
    }

    Ok(b'\x1b')
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
