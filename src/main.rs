use kirocode::{Editor, StdinRawMode};
use std::io::{stdout, BufWriter};

fn main() {
    let input = match StdinRawMode::new() {
        Ok(i) => i,
        Err(err) => panic!(err),
    };

    let out = stdout();
    let out = BufWriter::new(out.lock());

    Editor::new(input, out).edit();
}
