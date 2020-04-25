use kirocode::{StdinRawMode, KeySeq};
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
        if let Some(b) = input.read_byte().unwrap() {
            let input_seq = input.decode(b).unwrap();
            
            if input_seq.ctrl && input_seq.key == KeySeq::Key(b'q') {
                break;
            }
            
            let c = b as char;
            if input_seq.ctrl {
                // 制御文字かどうかを判定
                // 制御文字は画面に出力したくない印刷不可能な文字
                write!(out, "{}\r\n", b).unwrap();
            } else {
                write!(out, "{} ('{}')\r\n", b, c).unwrap();
            }
        }
    }
}
