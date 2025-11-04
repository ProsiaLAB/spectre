//! Readers and writers for the Leiden Atomic and Molecular Database (LAMDA)

use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use crate::errors::database::LAMDAError;
use crate::io::skip_line;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CollRate {
    pub temp: f64,
    pub rate: f64,
}

#[derive(Debug, Clone)]
pub struct ColliTransition {
    pub partner: String,
    pub id: usize,
    pub up: usize,
    pub low: usize,
    pub coll_rates: Vec<CollRate>,
}

#[derive(Debug, Clone)]
pub struct CollSet {
    pub temps: Vec<f64>,
    pub coll_transitions: Vec<ColliTransition>,
}

#[derive(Debug, Default, Clone)]
pub struct LAMDAData {
    pub name: String,
    pub weight: f64,
    pub levels: Vec<Level>,
    pub radset: Vec<RadTransition>,
    pub collsets: HashMap<String, CollSet>,
}

impl LAMDAData {
    pub fn from_reader<R: BufRead>(mut reader: R) -> Result<Self, LAMDAError> {
        let mut buf = String::new();

        // Molecule name
        skip_line(&mut reader, &mut buf)?;
        buf.clear();
        if reader.read_line(&mut buf)? == 0 {
            return Err(LAMDAError::ParseError("Missing molecule name".into()));
        }
        let molecule_name = buf.trim().to_string();

        // Molecular weight
        skip_line(&mut reader, &mut buf)?;
        buf.clear();
        if reader.read_line(&mut buf)? == 0 {
            return Err(LAMDAError::ParseError("Missing molecule weight".into()));
        }
        let molecule_weight: f64 = buf.trim().parse()?;

        // Number of energy levels
        buf.clear();
        reader.read_line(&mut buf)?;
        buf.clear();
        if reader.read_line(&mut buf)? == 0 {
            return Err(LAMDAError::ParseError("Missing level count".into()));
        }
        let level_count: usize = buf.trim().parse()?;

        let mut levels = Vec::with_capacity(level_count);

        // Energy levels
        skip_line(&mut reader, &mut buf)?;
        for _ in 0..level_count {
            buf.clear();
            if reader.read_line(&mut buf)? == 0 {
                return Err(LAMDAError::ParseError(
                    "Unexpected EOF in levels section".into(),
                ));
            }

            let mut fields = buf.split_whitespace();
            let id: usize = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing level ID".into()))?
                .parse()?;
            let energy: f64 = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing level energy".into()))?
                .parse()?;
            let weight: f64 = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing level weight".into()))?
                .parse()?;
            let j = fields
                .next()
                .map(|s| s.parse().unwrap_or(0)) // default to 0 if missing
                .unwrap_or(0);

            levels.push(Level {
                id,
                energy,
                weight,
                j,
            });
        }

        // Number of radiative transitions
        skip_line(&mut reader, &mut buf)?;
        buf.clear();
        if reader.read_line(&mut buf)? == 0 {
            return Err(LAMDAError::ParseError(
                "Missing radiative transition count".into(),
            ));
        }
        let rad_transition_count: usize = buf.trim().parse()?;

        let mut radset = Vec::with_capacity(level_count);

        // Radiative transitions
        skip_line(&mut reader, &mut buf)?;
        for _ in 0..rad_transition_count {
            buf.clear();
            if reader.read_line(&mut buf)? == 0 {
                return Err(LAMDAError::ParseError(
                    "Unexpected EOF in radiative transitions section".into(),
                ));
            }

            let mut fields = buf.split_whitespace();

            let id: usize = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing radiative transition ID".into()))?
                .parse()?;
            let upper_level: usize = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing upper level".into()))?
                .parse()?;
            let lower_level: usize = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing lower level".into()))?
                .parse()?;
            let einst_a: f64 = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing Einstein A coefficient".into()))?
                .parse()?;
            let freq: f64 = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing frequency".into()))?
                .parse()?;
            let energy: f64 = fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing energy".into()))?
                .parse()?;

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
        skip_line(&mut reader, &mut buf)?;
        buf.clear();
        if reader.read_line(&mut buf)? == 0 {
            return Err(LAMDAError::ParseError(
                "Missing collisional partner count".into(),
            ));
        }
        let colli_partner_count: usize = buf.trim().parse()?;

        let mut collsets = HashMap::new();

        for _ in 0..colli_partner_count {
            // Partner ID
            skip_line(&mut reader, &mut buf)?; // skip unused line
            buf.clear();
            reader.read_line(&mut buf)?;
            let partner_id = buf
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
            skip_line(&mut reader, &mut buf)?; // skip unused line
            buf.clear();
            reader.read_line(&mut buf)?;
            let colli_transition_count: usize = buf.trim().parse()?;

            // Number of collisional temperatures
            skip_line(&mut reader, &mut buf)?; // skip unused line
            buf.clear();
            reader.read_line(&mut buf)?;
            let _colli_temp_count: usize = buf.trim().parse()?;

            // Collisional temperatures
            skip_line(&mut reader, &mut buf)?; // skip unused line
            buf.clear();
            reader.read_line(&mut buf)?;
            let temps: Vec<f64> = buf
                .split_whitespace()
                .map(|x| {
                    x.parse::<f64>()
                        .map_err(|e| LAMDAError::ParseError(e.to_string()))
                })
                .collect::<Result<_, _>>()?;

            let mut coll_transitions = Vec::with_capacity(colli_transition_count);

            // Collisional transitions
            skip_line(&mut reader, &mut buf)?; // skip unused line
            for _ in 0..colli_transition_count {
                buf.clear();
                reader.read_line(&mut buf)?;
                let mut fields = buf.split_whitespace();

                let id: usize = fields
                    .next()
                    .ok_or_else(|| LAMDAError::ParseError("missing id".into()))?
                    .parse()?;
                let up: usize = fields
                    .next()
                    .ok_or_else(|| LAMDAError::ParseError("missing up".into()))?
                    .parse()?;
                let low: usize = fields
                    .next()
                    .ok_or_else(|| LAMDAError::ParseError("missing low".into()))?
                    .parse()?;

                let mut coll_rates = Vec::with_capacity(temps.len());
                for (i, rate_str) in fields.enumerate() {
                    coll_rates.push(CollRate {
                        temp: temps[i],
                        rate: rate_str.parse()?,
                    });
                }

                coll_transitions.push(ColliTransition {
                    partner: partner_name.to_string(),
                    id,
                    up,
                    low,
                    coll_rates,
                });
            }

            collsets.insert(
                partner_name.to_string(),
                CollSet {
                    temps,
                    coll_transitions,
                },
            );
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
