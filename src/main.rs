use kirocode::{self, Editor, StdinRawMode};
use std::io;
use std::process::exit;

fn edit() -> kirocode::Result<()> {
    // TODO: Read input from stdin before start
    let input = StdinRawMode::new()?.input_keys();
    Editor::new(input, io::stdout(), None)?.edit()
}

fn main() {
    if let Err(err) = edit() {
        eprintln!("Error: {}", err);
        exit(1);
    }
}
