use std::{
    fs::File,
    io::{Read, Write, BufWriter, BufReader},
    os::unix::io::FromRawFd,
    process,
};
use byteorder::{ReadBytesExt, WriteBytesExt};

const BUF_SIZE: usize = 16384;

fn wrapped_write(writer: &mut Write, c: u8) {
    match writer.write_u8(c) {
        Ok(_) => {}
        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
            process::exit(0);
        }
        Err(err) => {
            eprintln!("Error writing: {}", err);
            process::exit(1);
        }
    }
}

fn main() {
    let stdin = unsafe { File::from_raw_fd(0) };
    let mut reader = BufReader::with_capacity(BUF_SIZE, stdin);
    let stdout = unsafe { File::from_raw_fd(1) };
    let mut writer = BufWriter::with_capacity(BUF_SIZE, stdout);

    let mut isa_buf = vec![0u8; 106];

    match reader.read_exact(&mut isa_buf) {
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

    writer.write(&isa_buf).expect("Failed to write ISA segment");
    writer.write(b"\n").expect("Failed to write newline after ISA");
    let terminator: u8 = isa_buf[105];

    loop {
        match reader.read_u8() {
            Ok(x) => {
                wrapped_write(&mut writer, x);
                if x == terminator {
                    wrapped_write(&mut writer, 10);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                writer.flush().unwrap();
                process::exit(0);
            }
            Err(err) => {
                eprintln!("Error processing stream: {}", err);
                writer.flush().unwrap();
                process::exit(1);
            }
        }
    }
}
