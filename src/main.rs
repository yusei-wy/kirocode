use kirocode::{Editor, Error, Result, StdinRawMode};

use std::env;
use std::io::{self, BufWriter};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut filepath: Option<&str> = None;
    if args.len() >= 2 {
        filepath = Some(&args[1]);
    }
    if let Err(err) = edit(filepath) {
        die(err);
    }
}

fn edit(filepath: Option<&str>) -> Result<()> {
    let input = StdinRawMode::new(io::stdin())?.input_keys();
    let output = io::stdout();
    let output = BufWriter::new(output.lock());
    match filepath {
        Some(f) => Editor::open(f, input, output)?.edit(),
        _ => Editor::new(input, output)?.edit(),
    }
}

fn die(err: Error) {
    print!("\x1b[2J");
    print!("\x1b[H");
    eprintln!("{}", err);
}
