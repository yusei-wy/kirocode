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
    ob.ok_or(Error::InputReadByteError)
}

fn editor_move_cursor(b: u8) {}

fn is_ctrl(b: u8) -> bool {
    match b {
        0x00..=0x1f | 0x7f => true,
        _ => false,
    }
}

fn ctrl_key(c: char) -> u8 {
    c as u8 & 0x1f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ctrl() {
        assert!(!is_ctrl(32));
        assert!(!is_ctrl(126));
        assert!(!is_ctrl(128));
        assert!(is_ctrl(0));
        assert!(is_ctrl(30));
        assert!(is_ctrl(31));
        assert!(is_ctrl(127));
    }

    #[test]
    fn test_ctrl_key() {
        assert_eq!(ctrl_key('q'), 17);
    }
}
