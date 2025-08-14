use std::io::BufRead;
use std::io::Error;

pub fn skip_line<R: BufRead>(reader: &mut R, buf: &mut String) -> Result<(), Error> {
    buf.clear();
    reader.read_line(buf)?;
    Ok(())
}
