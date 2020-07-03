use kirocode::Result;

use std::io::{self, Read};
use std::os::unix::io::AsRawFd;

struct StdinRawMode {
    stdin: io::Stdin,
}

impl StdinRawMode {
    fn new() -> Result<Self> {
        use termios::*;
        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();
        let mut termios = Termios::from_fd(fd)?;

        // C/C++ でビットの NOT 演算子は '~'
        termios.c_lflag &= !(ECHO);

        tcsetattr(fd, TCSAFLUSH, &termios)?;

        Ok(Self { stdin })
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

fn main() {
    match StdinRawMode::new() {
        Ok(mut input) => loop {
            match input.read_byte() {
                Ok(ob) => {
                    if let Some(b) = ob {
                        let c = b as char;
                        if c == 'q' {
                            break;
                        }
                    }
                }
                Err(e) => print!("{}", e),
            }
        },
        Err(err) => eprintln!("{}", err),
    };
}
