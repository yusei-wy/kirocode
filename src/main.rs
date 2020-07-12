use kirocode::{Error, Result, StdinRawMode};

use std::io::{self, Write};

fn main() {
    let output = io::stdout();
    let mut output = output.lock();

    editor_refresh_screen(&mut output);
    output.flush().unwrap();

    match StdinRawMode::new() {
        Ok(mut input) => loop {
            editor_refresh_screen(&mut output);
            match editor_process_keypress(&mut input, &mut output) {
                Ok(ok) => {
                    if !ok {
                        write!(&mut output, "\x1b[2J").unwrap();
                        write!(&mut output, "\x1b[H").unwrap();
                        break;
                    }
                }
                Err(err) => die(err, &mut output),
            }
        },
        Err(err) => die(err, &mut output),
    };
}

fn die(err: Error, output: &mut io::StdoutLock) {
    write!(output, "\x1b[2J").unwrap();
    write!(output, "\x1b[H").unwrap();
    eprintln!("{}", err);
}

fn editor_refresh_screen(output: &mut io::StdoutLock) {
    write!(output, "\x1b[2J").unwrap();
    write!(output, "\x1b[H").unwrap();

    editor_draw_rows(output);

    write!(output, "\x1b[H").unwrap();

    output.flush().unwrap();
}

fn editor_draw_rows(output: &mut io::StdoutLock) {
    for _ in 0..24 {
        write!(output, "~\r\n").unwrap();
    }
}

fn editor_process_keypress(input: &mut StdinRawMode, output: &mut io::StdoutLock) -> Result<bool> {
    let b = editor_read_key(input)?;

    let c = b as char;
    if is_ctrl(b) {
        write!(output, "{}\r\n", b).unwrap();
    } else {
        write!(output, "{} ({})\r\n", b, c).unwrap();
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
    use crate::*;

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
