use spectre::errors::LAMDAError;
use spectre::lamda::LAMDAData;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), LAMDAError> {
    println!("Hello, LAMDA!");
    let file = File::open("data/molecule_13co.inp")?;
    let reader = BufReader::new(file);
    let data = LAMDAData::from_reader(reader)?;
    let collrates = data
        .collsets
        .values()
        .flat_map(|transitions| {
            transitions
                .iter()
                .flat_map(|t| t.coll_rate.iter().map(|cr| cr.rate))
        })
        .collect::<Vec<f64>>();
    let target_temp = 2.0;

    let rates_per_transition = data
        .collsets
        .values()
        .map(|transitions| {
            transitions
                .iter()
                .map(|t| {
                    t.coll_rate
                        .iter()
                        .find(|cr| cr.temp == target_temp)
                        .map(|cr| cr.rate)
                })
                .collect::<Vec<Option<f64>>>()
        })
        .collect::<Vec<Vec<Option<f64>>>>();
    dbg!(rates_per_transition);
    Ok(())
}
