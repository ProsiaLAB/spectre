//! Readers and writers for the Leiden Atomic and Molecular Database (LAMDA)

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::errors::database::LAMDAError;

#[derive(Debug)]
pub struct Level {
    /// ID of the level
    pub id: usize,
    /// Energy of the level in cm^-1
    pub energy: f64,
    ///  Statistical weight (degeneracy) of the level, g = (2J + 1) * symmetry_factor
    pub weight: f64,
    /// Total angular momentum quantum number of the level.
    pub j: usize,
}

#[derive(Debug)]
pub struct RadTransition {
    /// ID of the transition
    pub id: usize,
    /// Upper level ID
    pub up: usize,
    /// Lower level ID
    pub low: usize,
    /// Einstein A coefficient (s^-1)
    pub einst_a: f64,
    /// Frequency of the transition (GHz)
    pub freq: f64,
    /// Energy of the transition in (K)
    pub energy: f64,
}

#[derive(Debug)]
pub struct CollRate {
    pub temp: f64,
    pub rate: f64,
}

#[derive(Debug)]
pub struct ColliTransition {
    pub partner: String,
    pub id: usize,
    pub up: usize,
    pub low: usize,
    pub coll_rates: Vec<CollRate>,
}

#[derive(Debug)]
pub struct CollSet {
    pub temps: Vec<f64>,
    pub coll_transitions: Vec<ColliTransition>,
}

#[derive(Debug, Default)]
pub struct LAMDAData {
    pub name: String,
    pub weight: f64,
    pub levels: Vec<Level>,
    pub radset: Vec<RadTransition>,
    pub collsets: HashMap<String, CollSet>,
}

impl LAMDAData {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, LAMDAError> {
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        // Molecule name
        lines.next();
        let molecule_name = lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing molecule name".into()))??;

        // Molecular weight
        lines.next();
        let molecule_weight: f64 = lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing molecule weight".into()))??
            .parse()?;

        // Number of energy levels
        lines.next();
        let level_count: usize = lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing level count".into()))??
            .parse()?;

        let mut levels = Vec::new();

        // Energy levels
        lines.next();
        for line in lines.by_ref().take(level_count) {
            let line = line?;
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let id: usize = fields[0].parse()?;
            let energy: f64 = fields[1].parse()?;
            let weight: f64 = fields[2].parse()?;
            let j: usize = fields[3].parse()?;
            levels.push(Level {
                id,
                energy,
                weight,
                j,
            });
        }

        // Number of radiative transitions
        lines.next();
        let rad_transition_count: usize = lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing radiative transition count".into()))??
            .parse()?;

        let mut radset = Vec::new();

        // Radiative transitions
        lines.next();
        for line in lines.by_ref().take(rad_transition_count) {
            let line = line?;
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let id: usize = fields[0].parse()?;
            let upper_level: usize = fields[1].parse()?;
            let lower_level: usize = fields[2].parse()?;
            let einst_a: f64 = fields[3].parse()?;
            let freq: f64 = fields[4].parse()?;
            let energy: f64 = fields[5].parse()?;
            radset.push(RadTransition {
                id,
                up: upper_level,
                low: lower_level,
                einst_a,
                freq,
                energy,
            });
        }

        // Number of collisional partners
        lines.next();
        let colli_partner_count: usize = lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing collisional partner count".into()))??
            .parse()?;

        let mut collsets = HashMap::new();

        for _ in 0..colli_partner_count {
            // Partner ID
            lines.next();
            let partner_id_line = lines
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing partner ID".into()))??;
            let partner_id = partner_id_line
                .split_whitespace()
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing partner ID".into()))?;

            let partner_name = match partner_id {
                "1" => "H2",
                "2" => "p-H2",
                "3" => "o-H2",
                "4" => "e",
                "5" => "H",
                "6" => "He",
                "7" => "H+",
                _ => {
                    return Err(LAMDAError::ParseError(
                        "Invalid partner transition ID".into(),
                    ));
                }
            };

            // Number of collisional transitions
            lines.next();
            let colli_transition_count: usize = lines
                .next()
                .ok_or_else(|| {
                    LAMDAError::ParseError("Missing collisional transition count".into())
                })??
                .parse()?;

            // Number of collisional temperatures
            lines.next();
            let _colli_temp_count: usize = lines
                .next()
                .ok_or_else(|| {
                    LAMDAError::ParseError("Missing collisional temperature count".into())
                })??
                .parse()?;

            // Collisional temperatures
            lines.next();
            let temp_line = lines.next().ok_or_else(|| {
                LAMDAError::ParseError("Missing collisional temperatures".into())
            })??;
            let temps = temp_line
                .split_whitespace()
                .map(|x| {
                    x.parse::<f64>()
                        .map_err(|e| LAMDAError::ParseError(e.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let mut coll_transitions = Vec::new();

            // Collisional transitions
            lines.next();
            for line in lines.by_ref().take(colli_transition_count) {
                let line = line?;
                let fields = line.split_whitespace().collect::<Vec<_>>();
                let (transition_info, coll_rate_data) = fields.split_at(3);
                let id: usize = transition_info[0].parse()?;
                let up: usize = transition_info[1].parse()?;
                let low: usize = transition_info[2].parse()?;

                let mut coll_rates = Vec::new();

                for (i, &temp) in temps.iter().enumerate() {
                    let coll_rate = CollRate {
                        temp,
                        rate: coll_rate_data[i].parse()?,
                    };
                    coll_rates.push(coll_rate);
                }
                let coll_transition = ColliTransition {
                    partner: partner_name.to_string(),
                    id,
                    up,
                    low,
                    coll_rates,
                };
                coll_transitions.push(coll_transition);
            }
            let coll_set = CollSet {
                temps,
                coll_transitions,
            };
            collsets.insert(partner_name.to_string(), coll_set);
        }

        Ok(Self {
            name: molecule_name,
            weight: molecule_weight,
            levels,
            radset,
            collsets,
        })
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LAMDAError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }
}
