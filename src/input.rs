use crate::error::Result;

use std::fmt;
use std::io::{self, Read};
use std::ops::{Deref, DerefMut};
use std::os::unix::io::AsRawFd;
use std::str;

pub struct StdinRawMode<R: Read + AsRawFd> {
    stdin: R,
    org: termios::Termios,
}

impl<R> StdinRawMode<R>
where
    R: Read + AsRawFd,
{
    pub fn new(stdin: R) -> Result<Self> {
        use termios::*;
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

    pub fn input_keys(self) -> InputSequences<R> {
        InputSequences { stdin: self }
    }

    fn disable_raw_mode(&mut self) {
        use termios::*;
        tcsetattr(self.stdin.as_raw_fd(), termios::TCSAFLUSH, &self.org).unwrap();
    }
}

impl<R> Drop for StdinRawMode<R>
where
    R: Read + AsRawFd,
{
    fn drop(&mut self) {
        self.disable_raw_mode();
    }
}

impl<R> Deref for StdinRawMode<R>
where
    R: Read + AsRawFd,
{
    type Target = R;

    fn deref(&self) -> &R {
        &self.stdin
    }
}

impl<R> DerefMut for StdinRawMode<R>
where
    R: Read + AsRawFd,
{
    fn deref_mut(&mut self) -> &mut R {
        &mut self.stdin
    }
}

#[derive(PartialEq, Debug)]
pub enum KeySeq {
    Left,
    Right,
    Up,
    Down,
    Del,
    Home,
    End,
    PageUp,
    PageDown,
    Key(u8),
    Cursor(usize, usize), // (x, y)
    Unidentified,
}

impl fmt::Display for KeySeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KeySeq::*;
        match self {
            Left => write!(f, "LEFT"),
            Right => write!(f, "RIGHT"),
            Up => write!(f, "UP"),
            Down => write!(f, "DOWN"),
            Del => write!(f, "DEL"),
            Home => write!(f, "HOME"),
            End => write!(f, "END"),
            PageUp => write!(f, "PAGE_UP"),
            PageDown => write!(f, "PAGE_DOWN"),
            Key(b' ') => write!(f, "SPACE"),
            Key(b) if b.is_ascii_control() => write!(f, "\\x{:x}", b),
            Key(b) => write!(f, "{}", *b as char),
            Cursor(x, y) => write!(f, "CURSOR({},{})", x, y),
            Unidentified => write!(f, "UNKNOWN"),
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

impl fmt::Display for InputSeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            write!(f, "C-")?;
        }
        if self.alt {
            write!(f, "M-")?;
        }
        write!(f, "{}", self.key)
    }
}

pub struct InputSequences<R>
where
    R: Read + AsRawFd,
{
    stdin: StdinRawMode<R>,
}

impl<R> InputSequences<R>
where
    R: Read + AsRawFd,
{
    fn read_seq(&mut self) -> Result<InputSeq> {
        if let Some(b) = self.read_byte()? {
            self.decode(b)
        } else {
            Ok(InputSeq::new(KeySeq::Unidentified))
        }
    }

    fn read_byte(&mut self) -> Result<Option<u8>> {
        let mut one_byte: [u8; 1] = [0];
        Ok(if self.stdin.read(&mut one_byte)? == 0 {
            None
        } else {
            Some(one_byte[0])
        })
    }

    fn decode(&mut self, b: u8) -> Result<InputSeq> {
        use KeySeq::*;
        match b {
            // 制御文字
            // TODO: | 00011111 にしなくても動作するかチェック
            0x1b => self.decode_escape_sequence(),
            0x00..=0x1f | 0x7f => Ok(InputSeq::ctrl(Key(b & 0x1f))),
            0x20..=0x7e => Ok(InputSeq::new(Key(b))),
            _ => Ok(InputSeq::new(Unidentified)),
        }
    }

    fn decode_escape_sequence(&mut self) -> Result<InputSeq> {
        use KeySeq::*;

        match self.read_byte()? {
            Some(b'[') | Some(b'O') => {}
            Some(b) if b.is_ascii_control() => return Ok(InputSeq::new(Key(0x1b))),
            Some(b) => {}
            None => return Ok(InputSeq::new(Key(0x1b))),
        };

        let mut buf = vec![];
        let cmd = loop {
            if let Some(b) = self.read_byte()? {
                match b {
                    b'A' | b'B' | b'C' | b'D' | b'H' | b'F' | b'R' | b'~' => break b,
                    _ => buf.push(b),
                };
            } else {
                // 不明なエスケープシーケンスは無視する
                return Ok(InputSeq::new(Unidentified));
            }
        };

        let mut args = buf.split(|b| *b == b';');
        match cmd {
            b'R' => {
                let mut i = args.filter_map(parse_bytes_as_usize);
                match (i.next(), i.next()) {
                    (Some(x), Some(y)) => Ok(InputSeq::new(Cursor(x, y))),
                    _ => Ok(InputSeq::new(Unidentified)),
                }
            }

            b'~' => match args.next() {
                Some(b"1") | Some(b"7") => return Ok(InputSeq::new(Home)),
                Some(b"4") | Some(b"8") => return Ok(InputSeq::new(End)),
                Some(b"3") => return Ok(InputSeq::new(Del)),
                Some(b"5") => return Ok(InputSeq::new(PageUp)),
                Some(b"6") => return Ok(InputSeq::new(PageDown)),
                _ => return Ok(InputSeq::new(Unidentified)),
            },

            b'A' => return Ok(InputSeq::new(Up)),
            b'B' => return Ok(InputSeq::new(Down)),
            b'C' => return Ok(InputSeq::new(Right)),
            b'D' => return Ok(InputSeq::new(Left)),

            b'H' | b'F' => {
                let key = match cmd {
                    b'H' => Home,
                    b'F' => End,
                    _ => unreachable!(),
                };
                return Ok(InputSeq::new(key));
            }

            _ => unreachable!(),
        }
    }
}

fn parse_bytes_as_usize(b: &[u8]) -> Option<usize> {
    str::from_utf8(b).ok().and_then(|s| s.parse().ok())
}

impl<R> Iterator for InputSequences<R>
where
    R: Read + AsRawFd,
{
    type Item = Result<InputSeq>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.read_seq())
    }
}

pub struct DummyInputSequences(pub Vec<InputSeq>);

impl Iterator for DummyInputSequences {
    type Item = Result<InputSeq>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            Some(Ok(self.0.remove(0)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::io::RawFd;

    use KeySeq::*;

    struct DummyStdin(Vec<u8>);

    impl Read for DummyStdin {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            buf.as_mut().write(&self.0).unwrap();
            Ok(buf.len())
        }
    }

    impl AsRawFd for DummyStdin {
        fn as_raw_fd(&self) -> RawFd {
            0
        }
    }

    fn dummy_input_keys(buf: &[u8]) -> InputSequences<DummyStdin> {
        use termios::*;
        let stdin = DummyStdin(buf.to_vec());
        let fd = stdin.as_raw_fd();
        let org = Termios::from_fd(fd).unwrap();
        let stdin = StdinRawMode { stdin, org };
        stdin.input_keys()
    }

    #[test]
    fn test_read_byte() {
        let mut i = dummy_input_keys(b"");
        assert_eq!(i.read_byte().unwrap().unwrap(), 0);

        i = dummy_input_keys(b"a");
        assert_eq!(i.read_byte().unwrap().unwrap(), b'a');

        i = dummy_input_keys("b")
    }

    #[test]
    fn test_input_sequences_decode() {
        let mut i = dummy_input_keys(b"");
        assert_eq!(i.decode(0x80).unwrap().key, Unidentified);
        assert_eq!(i.decode(0x20).unwrap().key, Key(0x20));
        assert_eq!(i.decode(0x7e).unwrap().key, Key(0x7e));
    }

    #[test]
    fn test_input_seq() {
        let i1 = InputSeq::new(Key(b'a'));
        assert_eq!(i1.key, Key(b'a'));
        assert_eq!(i1.ctrl, false);
        assert_eq!(i1.alt, false);

        let i2 = InputSeq::ctrl(Key(b'a'));
        assert_eq!(i2.key, Key(b'a'));
        assert_eq!(i2.ctrl, true);
        assert_eq!(i2.alt, false);

        let i3 = InputSeq::alt(Key(b'a'));
        assert_eq!(i3.key, Key(b'a'));
        assert_eq!(i3.ctrl, false);
        assert_eq!(i3.alt, true);
    }

    #[test]
    fn test_parse_bytes_as_usize() {
        assert_eq!(parse_bytes_as_usize(b""), None);
        assert_eq!(parse_bytes_as_usize(b" "), None);
        assert_eq!(parse_bytes_as_usize(b"hogehoge"), None);
        assert_eq!(parse_bytes_as_usize(b"0"), Some(0));
        assert_eq!(parse_bytes_as_usize(b"1"), Some(1));
        assert_eq!(parse_bytes_as_usize(b"10"), Some(10));
    }
}
