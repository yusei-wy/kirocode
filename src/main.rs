use kirocode::{Error, Result, Screen, StdinRawMode};

use std::io::{self, BufWriter};

fn main() {
    if let Err(err) = edit() {
        die(err);
    }
}

fn edit() -> Result<()> {
    let mut input = StdinRawMode::new()?;
    let output = io::stdout();
    let output = BufWriter::new(output.lock());

    let mut screen = Screen::new(None, &mut input, output)?;
    screen.refresh()?;

    loop {
        screen.refresh()?;
        let ok = editor_process_keypress(&mut input)?;
        if !ok {
            screen.clear()?;
            break;
        }
    }

    Ok(())
}

fn die(err: Error) {
    print!("\x1b[2J");
    print!("\x1b[H");
    eprintln!("{}", err);
}

fn editor_process_keypress(input: &mut StdinRawMode) -> Result<bool> {
    let b = editor_read_key(input)?;

    let c = b as char;
    if is_ctrl(b) {
        print!("{}\r\n", b);
    } else {
        print!("{} ({})\r\n", b, c);
    }

    if b == ctrl_key('q') {
        return Ok(false);
    }

    Ok(true)
}

fn editor_read_key(input: &mut StdinRawMode) -> Result<u8> {
    let ob = input.read_byte()?;
    match ob {
        Some(b) => Ok(b),
        None => Err(Error::InputReadByteError),
    }
}

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
