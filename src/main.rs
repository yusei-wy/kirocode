use std::io::Error;
use std::io::{self, Read};
use std::os::unix::io::{AsRawFd, RawFd};
use std::process::exit;

pub struct StdinRawMode {
    stdin: io::Stdin,
    fd: RawFd,
    orig: termios::Termios,
}

impl StdinRawMode {
    pub fn new() -> Result<StdinRawMode, Error> {
        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();
        let orig = termios::Termios::from_fd(fd)?;

        Ok(StdinRawMode { stdin, fd, orig })
    }

    pub fn enable_raw_mode(&self) {
        use termios::*;

        let mut termios = Termios::from_fd(self.fd).unwrap();

        termios.c_lflag &= !(ECHO | ICANON);

        tcsetattr(self.fd, TCSAFLUSH, &termios).unwrap();
    }

    pub fn disable_raw_mode(&self) {
        termios::tcsetattr(self.fd, termios::TCSAFLUSH, &self.orig).unwrap();
    }

    pub fn read_byte(&mut self) -> Result<Option<u8>, Error> {
        let mut one_byte: [u8; 1] = [0];
        Ok(if self.stdin.read(&mut one_byte)? == 0 {
            None
        } else {
            Some(one_byte[0])
        })
    }
}

fn main() {
    let mut input = match StdinRawMode::new() {
        Ok(i) => i,
        Err(err) => panic!(err),
    };
    input.enable_raw_mode();

    loop {
        match input.read_byte() {
            Ok(b) => {
                if b.unwrap() == b'q' {
                    break;
                }
                print!("{}", b.unwrap());
            }
            _ => break,
        };
    }

    input.disable_raw_mode();
}
