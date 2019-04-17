use aho_corasick::AhoCorasick;

use std::io::{Read, Write, stdin, stdout};
use std::process;
use std::str;

fn aho () {
    let mut rdr = stdin();
    let mut isa_buf = vec![0u8; 106];

    match rdr.read_exact(&mut isa_buf) {
        Ok(_) => {}
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            eprintln!("ISA segment is too short.");
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Error processing stream: {}", err);
            process::exit(1);
        }

    }

    stdout().write(&isa_buf).expect("Failed to write ISA segment");

    let terminator = str::from_utf8(&isa_buf[105..106]).unwrap();
    let replacement = format!("{}\n", terminator);
    let patterns = &[terminator];
    let replace_with = &[replacement];

    let ac = AhoCorasick::new(patterns);

    match ac.stream_replace_all(rdr, &mut stdout(), replace_with) {
        Ok(_) => {}
        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Error processing stream: {}", err);
            process::exit(1);
        }
    }
}

fn main() {
    aho();
}
