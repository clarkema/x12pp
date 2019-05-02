# x12pp

`x12pp` is a CLI pretty-printer for X12 EDI files.

X12 is an arcane format consisting of a fixed-length header followed by a series
of segments, each separated by a segment terminator character.

These segments are generally not separated by newlines, so extracting a range of
lines from a file or taking a peek at the start using the usual Unix toolbox
becomes unnecessarily painful.

Of course, you could split the lines using `sed -e 's/~/~\n/g'` and get on with
your day, but:

  1. although the `~` is the traditional and most widely-used segment terminator
     it's not required -- each X12 file specifies its own terminators as part of
    the header.
  2. using `sed` or `perl` would mean I wouldn't have a chance to explore fast
     stream processing in Rust.

So here we are.

## Installation

### Homebrew

```
$ brew tap clarkema/nomad
$ brew install x12pp
```

### With cargo

```
$ cargo install x12pp
```

### From source

x12pp is written in Rust, so you'll need an up-to-date Rust installation in
order to build it from source.  The result is a statically-compiled binary at
`target/release/x12pp`, which you can copy wherever you need.

```
$ git clone https://github.com/clarkema/x12pp
$ cd x12pp
$ cargo build --release
$ ./target/release/x12pp --version
```

## Usage

```
$ x12pp < FILE > NEWFILE
$ x12pp FILE -o NEWFILE

# Strip newlines out instead with:
$ x12pp --uglify FILE
```

See manpage or `--help` for more.

## Benchmarks

All tests were performed on an Intel Core i9-7940X, using a 1.3G X12 test file
located on a RAM disk.  In each case, shell redirection was used to
pipe the file through the test command and into `/dev/null` in order to get
as close as possible to measuring pure processing time.  For example:

`$ time sed -e 's/~/~\n/g' < test-file > /dev/null`

| Tool        | Command                       | Terminator detection | Pre-wrapped? | SIGPIPE? | Time  |
|-------------|-------------------------------|----------------------|--------------|----------|-------|
| x12pp       | `x12pp`                       | ✓                    | ✓            | ✓        | 1.3s  |
| GNU sed 4.7 | `sed -e s/~/~\n/g`            | ✗                    | ✗            | ✗        | 7.6s  |
| perl 5.28.2 | `perl -pe 's/~[\r\n]*/~\n/g'` | ✗                    | ✓ but slower | ✗        | 8.5s  |
| edicat      | `edicat`                      | ✓                    | ✓            | ✓        | 7m41s |

### Notes

1. 'SIGPIPE' refers to whether a command can return a partial result without
   having to process the entire input.  One of the motivations for `x12pp` was
   to be able to run `x12pp < FILE | head -n 100` without having to plough
   through a multi-gigabyte file.
2. Of course you could write a Perl script that _did_ correctly read the
   segment terminator before processing the rest of the file.
3. Perl produces the correct output with input data that is already wrapped,
   but _much_ slower; around 24 seconds compared to 8.5.
4. See https://github.com/notpeter/edicat for edicat
