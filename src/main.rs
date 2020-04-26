use kirocode::{InputSeq, KeySeq, Result, StdinRawMode};
use std::io::{self, stdout, BufWriter, Write};

fn editor_process_keypress(out: &mut BufWriter<io::StdoutLock>, input: &mut StdinRawMode) -> bool {
    let input_seq = editor_read_key(input).unwrap();

    if input_seq.ctrl && input_seq.key == KeySeq::Key(b'q') {
        // exit すると StdRawMode の Drop が呼ばれない
        return false;
    }

    true
}

fn editor_read_key(input: &mut StdinRawMode) -> Result<InputSeq> {
    let b = input.read_byte().unwrap();
    input.decode(b.unwrap())
}

fn editor_refresh_screen(out: &mut BufWriter<io::StdoutLock>) {
    write!(out, "{}", "\x1b[2J").unwrap();
    write!(out, "{}", "\x1b[H").unwrap();
}

fn main() {
    let mut input = match StdinRawMode::new() {
        Ok(i) => i,
        Err(err) => panic!(err),
    };

    input.enable_raw_mode();

    let out = stdout();
    let mut out = BufWriter::new(out.lock());
    loop {
        editor_refresh_screen(&mut out);
        if !editor_process_keypress(&mut out, &mut input) {
            break;
        }
    }
}
