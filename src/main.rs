use std::{
    fs::File,
    io,
    io::{Read, Write, BufWriter, BufReader, stdout},
    os::unix::io::FromRawFd,
    process,
};
use byteorder::{WriteBytesExt}; // for write_u8

const BUF_SIZE: usize = 16384;
const NL: u8 = 13;

fn run() -> io::Result<()> {
    let stdin = unsafe { File::from_raw_fd(0) };
    let mut reader = BufReader::with_capacity(BUF_SIZE, stdin);
    let stdout = stdout();
    let mut writer = BufWriter::with_capacity(BUF_SIZE, stdout.lock());

    let mut buf = vec![0u8; 106];

    reader.read_exact(&mut buf).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            io::Error::new(e.kind(), "ISA segment is too short.")
        }
        else{
            e
        }
    })?;

    writer.write_all(&buf)?;
    writer.write_u8(b'\n')?;

    let terminator: u8 = buf[105];

    buf = vec![0u8; BUF_SIZE];
    let mut gobble_mode: bool = true;
    let mut i: usize;
    let mut start: usize;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { return Ok(()) }
        i = 0;
        start = 0;

        if gobble_mode {
            while (i < n) && (buf[i] == 13) {
                i += 1;
            }
            i += 1;
            start = i;
            gobble_mode = false;
        }

        'segment: loop {
            if i == n {
                writer.write_all(&buf[start..i])?;
                break;
            }
            if buf[i] == terminator {
                writer.write_all(&buf[start..=i])?;
                writer.write_u8(b'\n')?;

                // If we've found a segment terminator, we need to discard any
                // newlines that follow it.  Unfortunately these might span the
                // boundary of the buffer and into the next read.
                //
                // 1. Check whether we've hit the end of the buffer.  If so,
                //    set a flag to continue gobbling newlines and read some
                //    more data.
                // 2. Otherwise, if we've found a newline, advance beyond it.
                // 3. And if not, break back into normal scanning mode
                i += 1;
                loop {
                    if i == n {
                        gobble_mode = true;
                        break 'segment;
                    }
                    if buf[i] == NL {
                        i += 1;
                    }
                    else {
                        break;
                    }
                }
                start = i + 1;
            }
            i += 1;
        }
    }
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {}
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}
