use kirocode::StdinRawMode;
use std::io::{stdout, Write};

fn main() {
    let mut input = match StdinRawMode::new() {
        Ok(i) => i,
        Err(err) => panic!(err),
    };
    input.enable_raw_mode();

    let out = stdout();
    let mut out = out.lock();
    loop {
        match input.read_byte() {
            Ok(b) => {
                if b.unwrap() == b'q' {
                    break;
                }
                let b = b.unwrap();
                let c = b as char;
                if c.is_ascii_control() {
                    // 制御文字かどうかを判定
                    // 制御文字は画面に出力したくない印刷不可能な文字
                    write!(out, "{}\r\n", b).unwrap();
                } else {
                    write!(out, "{} ('{}')\r\n", b, c).unwrap();
                }
            }
            _ => break,
        };
    }
}
