use spectre::errors::LAMDAError;
use spectre::lamda::LAMDAData;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), LAMDAError> {
    println!("Hello, LAMDA!");
    let file = File::open("data/molecule_13co.inp")?;
    let reader = BufReader::new(file);
    let data = LAMDAData::from_reader(reader)?;
    let upper_levels = data
        .coll_transitions
        .values()
        .flat_map(|transitions| transitions.iter().map(|t| t.coll_rate))
        .collect::<Vec<_>>();
    dbg!(&upper_levels);
    Ok(())
}
