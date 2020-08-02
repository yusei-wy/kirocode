use kirocode::{Editor, Error, Result};

use std::env;
use std::io::{self, BufWriter};

fn main() {
    if let Err(err) = edit() {
        die(err);
    }
}

fn edit() -> Result<()> {
    let output = io::stdout();
    let output = BufWriter::new(output.lock());

    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        Editor::open(&args[1], output)?.edit()
    } else {
        Editor::new(output)?.edit()
    }
}

fn die(err: Error) {
    print!("\x1b[2J");
    print!("\x1b[H");
    eprintln!("{}", err);
}
