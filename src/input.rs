use crate::error::Result;
use std::io::{self, Read};
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
}

impl Drop for StdinRawMode {
    fn drop(&mut self) {
        // Restore original terminal mode
        termios::tcsetattr(self.fd, termios::TCSAFLUSH, &self.orig).unwrap();
    }
}
