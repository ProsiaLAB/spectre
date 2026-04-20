use std::io::BufRead;
use std::io::Error;

/// Skip a line in a [`BufRead`] reader, consuming the line and returning an empty string.
///
/// # Errors
/// Returns an error if the reader fails to read a line.
pub fn skip_line<R: BufRead>(reader: &mut R, buf: &mut String) -> Result<(), Error> {
    buf.clear();
    reader.read_line(buf)?;
    Ok(())
}
