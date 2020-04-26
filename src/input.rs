use crate::error::{Result, Error};
use std::fmt;
use std::io::{self, Read};
use std::ops::{Deref, DerefMut};
use std::os::unix::io::{AsRawFd, RawFd};

pub struct StdinRawMode {
    stdin: io::Stdin,
    fd: RawFd,
    orig: termios::Termios,
}

impl StdinRawMode {
    pub fn new() -> Result<StdinRawMode> {
        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();
        let orig = termios::Termios::from_fd(fd)?;
        Ok(StdinRawMode { stdin, fd, orig })
    }

    pub fn enable_raw_mode(&self) {
        use termios::*;

        let mut termios = Termios::from_fd(self.fd).unwrap();

        termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        termios.c_oflag &= !(OPOST);
        termios.c_cflag &= !(CS8);
        termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 1;

        tcsetattr(self.fd, TCSAFLUSH, &termios).unwrap();
    }

    pub fn read_byte(&mut self) -> Result<Option<u8>> {
        let mut one_byte: [u8; 1] = [0];
        Ok(if self.stdin.read(&mut one_byte)? == 0 {
            None
        } else {
            Some(one_byte[0])
        })
    }

    pub fn decode(&mut self, b: u8) -> Result<InputSeq> {
        use KeySeq::*;
        
        match b {
            // C0 control characters
            0x00..=0x1f => match b {
                // 0x00~0x1f keys area ascii keys with ctrl. Ctrl mod masks key with 0b11111.
                // Here unmask it with 0b1100000. It only works with 0x61~0x7f.
                _ => Ok(InputSeq::ctrl(Key(b | 0b0110_0000))),
            },
            0x20..=0x7f => Ok(InputSeq::new(Key(b))),
            _ => Err(Error::UnexpectedError),
        }
    }
}

impl Drop for StdinRawMode {
    fn drop(&mut self) {
        // Restore original terminal mode
        termios::tcsetattr(self.fd, termios::TCSAFLUSH, &self.orig).unwrap();
    }
}

impl Deref for StdinRawMode {
    type Target = io::Stdin;

    fn deref(&self) -> &Self::Target {
        &self.stdin
    }
}

impl DerefMut for StdinRawMode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stdin
    }
}

#[derive(PartialEq, Debug)]
pub enum KeySeq {
    Unidentified,
    Key(u8), // Char code and ctrl mod
}

impl fmt::Display for KeySeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KeySeq::*;
        match self {
            Unidentified => write!(f, "UNKNOWN"),
            Key(b) if b.is_ascii_control() => write!(f, "\\x{:x}", b),
            Key(b) => write!(f, "{}", *b as char),
        }
    }
}

pub struct InputSeq {
    pub key: KeySeq,
    pub ctrl: bool,
    pub alt: bool,
}

impl InputSeq {
    pub fn new(key: KeySeq) -> Self {
        Self {
            key,
            ctrl: false,
            alt: false,
        }
    }

    pub fn ctrl(key: KeySeq) -> Self {
        Self {
            key,
            ctrl: true,
            alt: false,
        }
    }

    pub fn alt(key: KeySeq) -> Self {
        Self {
            key,
            ctrl: false,
            alt: true,
        }
    }
}
