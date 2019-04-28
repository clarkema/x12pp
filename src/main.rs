use byteorder::WriteBytesExt; // for write_u8;
#[macro_use]
extern crate clap;
use memchr::memchr;
use std::{
    env,
    fs::File,
    io,
    io::{BufReader, BufWriter, Read, Write},
    os::unix::io::FromRawFd,
    process,
};

const BUF_SIZE: usize = 16384;
const NL: u8 = 10;
const CR: u8 = 13;

fn run(input_path: &str, output_path: &str, uglify: bool) -> io::Result<()> {
    let mut reader = if input_path == "-" {
        let stdin = unsafe { File::from_raw_fd(0) };
        BufReader::with_capacity(BUF_SIZE, stdin)
    } else {
        BufReader::with_capacity(
            BUF_SIZE,
            File::open(input_path).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("Failed to open '{}': {}", input_path, e),
                )
            })?,
        )
    };
    let mut writer = if output_path == "-" {
        let stdout = unsafe { File::from_raw_fd(1) };
        BufWriter::with_capacity(BUF_SIZE, stdout)
    } else {
        BufWriter::with_capacity(
            BUF_SIZE,
            File::create(output_path).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("Failed to create '{}': {}", output_path, e),
                )
            })?,
        )
    };

    let mut buf = vec![0u8; 106];

    reader.read_exact(&mut buf).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            io::Error::new(e.kind(), "ISA segment is too short.")
        } else {
            e
        }
    })?;

    let terminator: u8 = buf[105];
    writer.write_all(&buf)?;
    if !uglify {
        writer.write_u8(NL)?;
    }

    buf = vec![0u8; BUF_SIZE];
    let mut gobble_mode: bool = true;
    let mut i: usize;
    let mut start: usize;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        i = 0;
        start = 0;

        if gobble_mode {
            while (i < n) && (buf[i] == NL || buf[i] == CR) {
                i += 1;
            }
            start = i;
            gobble_mode = false;
        }

        'segment: while start < n {
            match memchr(terminator, &buf[start..n]) {
                Some(offset) => {
                    writer.write_all(&buf[start..=start + offset])?;
                    if !uglify {
                        writer.write_u8(NL)?;
                    }

                    // If we've found a segment terminator, we need to discard any
                    // newlines that follow it.  Unfortunately these might span the
                    // boundary of the buffer and into the next read.
                    //
                    // 1. Check whether we've hit the end of the buffer.  If so,
                    //    set a flag to continue gobbling newlines and read some
                    //    more data.
                    // 2. Otherwise, if we've found a newline, advance beyond it.
                    // 3. And if not, break back into normal scanning mode
                    let mut i = start + offset + 1;
                    loop {
                        if i == n {
                            gobble_mode = true;
                            break 'segment;
                        }
                        if buf[i] == NL || buf[i] == CR {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    start = i;
                }
                None => {
                    writer.write_all(&buf[start..n])?;
                    break;
                }
            }
        }
    }
}

fn main() {
    let mut input_path = "-";
    let mut output_path = "-";
    let mut uglify = false;
    let matches;

    if env::args().len() > 1 {
        matches = clap_app!(x12pp =>
            (version: "0.1.0")
            (author: "Mike Clarke <clarkema@clarkema.org>")
            (about: "X12 pretty-printer")
            (@arg INPUT: "Input file.  Omit or use '-' for STDIN")
            (@arg output: -o --output +takes_value "Output file.")
            (@arg uglify: -u --uglify "Uglify mode: strip post-segment newlines")
        )
        .get_matches();

        input_path = matches.value_of("INPUT").unwrap_or("-");
        output_path = matches.value_of("output").unwrap_or("-");
        uglify = matches.is_present("uglify");
    }

    match run(input_path, output_path, uglify) {
        Ok(_) => {}
        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}
