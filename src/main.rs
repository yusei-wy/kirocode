use kirocode::Result;

use std::io::{self, Read};
use std::os::unix::io::AsRawFd;

struct StdinRawMode {
    stdin: io::Stdin,
    org: termios::Termios,
}

impl StdinRawMode {
    fn new() -> Result<Self> {
        use termios::*;
        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();
        let mut termios = Termios::from_fd(fd)?;
        let org = termios;

        // C/C++ でビットの NOT 演算子は '~'
        termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        termios.c_oflag &= !(OPOST);
        termios.c_cflag &= !(CS8);
        termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 1;

        tcsetattr(fd, TCSAFLUSH, &termios)?;

        Ok(Self { stdin, org })
    }

    fn read_byte(&mut self) -> Result<Option<u8>> {
        let mut one_byte: [u8; 1] = [0];
        Ok(if self.stdin.read(&mut one_byte)? == 0 {
            None
        } else {
            Some(one_byte[0])
        })
    }
}

impl Drop for StdinRawMode {
    fn drop(&mut self) {
        use termios::*;
        tcsetattr(self.stdin.as_raw_fd(), termios::TCSAFLUSH, &self.org).unwrap();
    }
}

fn main() {
    match StdinRawMode::new() {
        Ok(mut input) => loop {
            let mut b: u8 = 0;
            match input.read_byte() {
                Ok(ob) => {
                    if let Some(_b) = ob {
                        b = _b;
                        let c = b as char;
                        if is_ctrl(b) {
                            print!("{}\r\n", b);
                        } else {
                            print!("{} ({})\r\n", b, c)
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    break;
                }
            }

            if b == ctrl_key('q') {
                break;
            }
        },
        Err(err) => eprintln!("{}", err),
    };
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
