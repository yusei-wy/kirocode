use crate::error::Result;

use std::io::{self, Read};
use std::os::unix::io::AsRawFd;

pub struct StdinRawMode {
    stdin: io::Stdin,
    org: termios::Termios,
}

impl StdinRawMode {
    pub fn new() -> Result<Self> {
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
        use termios::*;
        tcsetattr(self.stdin.as_raw_fd(), termios::TCSAFLUSH, &self.org).unwrap();
    }
}
