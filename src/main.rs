use kirocode::{Editor, Error, Result};

use std::io::{self, BufWriter};

fn main() {
    if let Err(err) = edit() {
        die(err);
    }
}

fn edit() -> Result<()> {
    let output = io::stdout();
    let output = BufWriter::new(output.lock());
    let mut editor = Editor::open(output)?;
    editor.edit()
}

fn die(err: Error) {
    print!("\x1b[2J");
    print!("\x1b[H");
    eprintln!("{}", err);
}
