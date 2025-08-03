//! Example of reading a LAMDA database

use spectre::errors::database::LAMDAError;
use spectre::lamda::LAMDAData;

fn main() -> Result<(), LAMDAError> {
    let molecule = LAMDAData::from_path(
        "/Users/kmaitreys/Documents/ProsiaLAB/spectre/data/molecule_13co.inp",
    )?;

    println!("{:?}", molecule);

    Ok(())
}
