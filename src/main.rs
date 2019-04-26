use std::{
    fs::File,
    io::{Read, Write, BufWriter, BufReader, stdin, stdout},
    os::unix::io::FromRawFd,
    process,
};
use byteorder::{ReadBytesExt, WriteBytesExt};

const BUF_SIZE: usize = 16384;

macro_rules! wrapped_write {
    ($writer:expr, $b:expr) => (
        match $writer.write_u8($b) {
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                process::exit(0);
            }
            Err(err) => {
                eprintln!("Error writing: {}", err);
                process::exit(1);
            }
        }
    );
}

fn main() {
    let stdin = unsafe { File::from_raw_fd(0) };
    let mut reader = BufReader::with_capacity(BUF_SIZE, stdin);
    let stdout = stdout();
    let mut writer = BufWriter::with_capacity(BUF_SIZE, stdout.lock());

    let mut buf = vec![0u8; 106];

    match reader.read_exact(&mut buf) {
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

    writer.write(&buf).expect("Failed to write ISA segment");
    wrapped_write!(writer, b'\n');

    let terminator: u8 = buf[105];

    let mut gobble_mode: bool = true;
    let mut buf = vec![0u8; BUF_SIZE];
    let mut i: usize;
    let mut start: usize;
    loop {
        match reader.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    writer.flush().unwrap();
                    process::exit(0);
                }
                i = 0;
                start = 0;

                if gobble_mode {
                    while (i < n) && (buf[i] == 13) {
                        i = i + 1;
                    }
                    i = i + 1;
                    start = i;
                    gobble_mode = false;
                }

                'outer: loop {
                    if i == n {
                        writer.write(&buf[start..i]);
                        break;
                    }
                    if buf[i] == terminator {
                        writer.write(&buf[start..i + 1]);
                        wrapped_write!(writer, b'\n');

                        /*
                         * If we've found a segment terminator, we need to
                         * discard any newlines that follow it.  Unfortunately
                         * these might span the boundary of the buffer and into
                         * the next read.
                         *
                         * 1. Check whether we've hit the end of the buffer.
                         *    If so, set a flag to continue gobbling newlines
                         *    and read some more data.
                         * 2. Otherwise, if we've found a newline, advance
                         *    beyond it.
                         * 3. And if not, break back into normal scanning mode
                         */
                        i = i + 1;
                        loop {
                            if i == n {
                                gobble_mode = true;
                                break 'outer;
                            }
                            if buf[i] == 13 {
                                i = i + 1;
                            }
                            else {
                                break;
                            }
                        }

                        start = i + 1;
                    }
                    i = i + 1;
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
