use spectre::errors::LAMDAError;
use spectre::lamda::LAMDAData;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), LAMDAError> {
    println!("Hello, LAMDA!");
    let file = File::open("data/molecule_13co.inp")?;
    let reader = BufReader::new(file);
    let data = LAMDAData::from_reader(reader)?;
    // dbg!(data);
    Ok(())
}
