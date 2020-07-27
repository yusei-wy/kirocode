use kirocode::{edit, Error};

fn main() {
    if let Err(err) = edit() {
        die(err);
    }
}

fn die(err: Error) {
    print!("\x1b[2J");
    print!("\x1b[H");
    eprintln!("{}", err);
}
