# x12pp

`x12pp` is a CLI pretty-printer for X12 EDI files.

X12 is an arcane format consisting of a fixed-length header
followed by a series of segments, each separated by a segment
terminator character.

These segments are generally not separated by newlines, so
extracting a range of lines from a file or taking a peek at
the start using the usual Unix toolbox becomes unnecessarily
painful.

Of course, you could split the lines using `sed -e 's/~/~\n'`
and get on with your day, but:

  1. although the `~` is the traditional and most widely-used
     segment terminator it's not required -- each X12 file
     specifies its own terminators as part of the header.
  2. using `sed` or `perl` would mean I wouldn't have a chance
     to explore fast stream processing in Rust.
