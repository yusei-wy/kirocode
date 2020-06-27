use kirocode::Result;

use std::io::{self, Read};

fn main() {
    let mut stdin = io::stdin();
    loop {
        match read_byte(&mut stdin) {
            Ok(ob) => {},
            Err(e) => print!("{}", e),
        }
    }
}

fn read_byte(stdin: &mut io::Stdin) -> Result<Option<u8>> {
    let mut one_byte: [u8; 1] = [0];
    Ok(if stdin.read(&mut one_byte)? == 0 {
        None
    } else {
        Some(one_byte[0])
    })
}
