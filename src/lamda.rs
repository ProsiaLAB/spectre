//! Readers and writers for the Leiden Atomic and Molecular Database (LAMDA)

use std::collections::HashMap;
use std::io::BufReader;
use std::io::{BufRead, Read};

use crate::errors::LAMDAError;

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
pub struct ColliTransition {
    pub id: usize,
    pub up: usize,
    pub low: usize,
    pub temps: Vec<f64>,
    pub coll_rate: Vec<(f64, f64)>,
}

#[derive(Debug, Default)]
pub struct LAMDAData {
    pub levels: Vec<Level>,
    pub rad_transitions: Vec<RadTransition>,
    pub coll_transitions: HashMap<String, Vec<ColliTransition>>,
}

impl LAMDAData {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, LAMDAError> {
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        lines
            .nth(1)
            .ok_or_else(|| LAMDAError::ParseError("Missing molecule header".into()))??;
        lines
            .nth(1)
            .ok_or_else(|| LAMDAError::ParseError("Missing weight".into()))??;

        let level_count: usize = lines
            .nth(1)
            .ok_or_else(|| LAMDAError::ParseError("Missing energy levels count".into()))??
            .parse()?;

        lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing energy level header".into()))??;

        let levels = (0..level_count)
            .map(|_| Self::parse_level(&mut lines))
            .collect::<Result<Vec<_>, _>>()?;

        let transition_count: usize = lines
            .nth(1)
            .ok_or_else(|| LAMDAError::ParseError("Missing radiative transition count".into()))??
            .parse()?;

        lines.next().ok_or_else(|| {
            LAMDAError::ParseError("Missing radiative transition header".into())
        })??;

        let rad_transitions = (0..transition_count)
            .map(|_| Self::parse_rad_transition(&mut lines))
            .collect::<Result<Vec<_>, _>>()?;

        let colli_partner_count: usize = lines
            .nth(1)
            .ok_or_else(|| {
                LAMDAError::ParseError("Missing collisional transition partner count".into())
            })??
            .parse()?;

        let mut coll_transitions = HashMap::new();
        for _ in 0..colli_partner_count {
            let partner_info = lines.nth(1).ok_or_else(|| {
                LAMDAError::ParseError("Missing partner transition header".into())
            })??;
            let partner_id = partner_info
                .split_whitespace()
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing partner transition ID".into()))?;
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
            let colli_transitions_count: usize = lines
                .nth(1)
                .ok_or_else(|| {
                    LAMDAError::ParseError("Missing collisional transition count".into())
                })??
                .parse()?;
            let _: usize = lines
                .nth(1)
                .ok_or_else(|| {
                    LAMDAError::ParseError("Missing collisional transition header".into())
                })??
                .parse()?;
            let colli_transitions_temps = lines
                .nth(1)
                .ok_or_else(|| {
                    LAMDAError::ParseError("Missing collisional transition temperatures".into())
                })??
                .split_whitespace()
                .map(|x| {
                    x.parse::<f64>()
                        .map_err(|e| LAMDAError::ParseError(e.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;

            lines.next().ok_or_else(|| {
                LAMDAError::ParseError("Missing collisional transition data".into())
            })??;
            let colli_transitions = (0..colli_transitions_count)
                .map(|_| Self::parse_colli_transition(&mut lines, &colli_transitions_temps))
                .collect::<Result<Vec<_>, _>>()?;

            coll_transitions.insert(partner_name.to_string(), colli_transitions);
        }

        Ok(Self {
            levels,
            rad_transitions,
            coll_transitions,
        })
    }

    fn parse_level<I>(lines: &mut I) -> Result<Level, LAMDAError>
    where
        I: Iterator<Item = Result<String, std::io::Error>>,
    {
        let line = lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing energy level".into()))??;
        let mut fields = line.split_whitespace();

        Ok(Level {
            id: fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing level ID".into()))?
                .parse()?,
            energy: fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing level energy (cm^-1)".into()))?
                .parse()?,
            weight: fields
                .next()
                .ok_or_else(|| {
                    LAMDAError::ParseError("Missing level statistical weight (degeneracy)".into())
                })?
                .parse()?,
            j: fields
                .next()
                .ok_or_else(|| {
                    LAMDAError::ParseError(
                        "Missing level total angular momentum quantum number".into(),
                    )
                })?
                .parse()?,
        })
    }

    fn parse_rad_transition<I>(lines: &mut I) -> Result<RadTransition, LAMDAError>
    where
        I: Iterator<Item = Result<String, std::io::Error>>,
    {
        let line = lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing radiative transition".into()))??;
        let mut fields = line.split_whitespace();

        Ok(RadTransition {
            id: fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing transition ID".into()))?
                .parse()?,
            up: fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing upper level ID".into()))?
                .parse()?,
            low: fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing lower level ID".into()))?
                .parse()?,
            einst_a: fields
                .next()
                .ok_or_else(|| {
                    LAMDAError::ParseError("Missing Einstein A coefficient (s^-1)".into())
                })?
                .parse()?,
            freq: fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing transition frequency (GHz)".into()))?
                .parse()?,
            energy: fields
                .next()
                .ok_or_else(|| LAMDAError::ParseError("Missing energy".into()))?
                .parse()?,
        })
    }

    fn parse_colli_transition<I>(
        lines: &mut I,
        temps: &[f64],
    ) -> Result<ColliTransition, LAMDAError>
    where
        I: Iterator<Item = Result<String, std::io::Error>>,
    {
        let line = lines
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing collisional transition".into()))??;

        let mut fields = line.split_whitespace();

        let id = fields
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing transition ID".into()))?
            .parse()
            .map_err(|e| LAMDAError::ParseError(format!("Invalid transition ID: {}", e)))?;

        let up = fields
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing upper level ID".into()))?
            .parse()
            .map_err(|e| LAMDAError::ParseError(format!("Invalid upper level ID: {}", e)))?;

        let low = fields
            .next()
            .ok_or_else(|| LAMDAError::ParseError("Missing lower level ID".into()))?
            .parse()
            .map_err(|e| LAMDAError::ParseError(format!("Invalid lower level ID: {}", e)))?;

        let mut coll_rate = Vec::new();

        for (i, field) in fields.enumerate() {
            let rate: f64 = field.parse()?;
            coll_rate.push((rate, temps[i]));
        }

        Ok(ColliTransition {
            id,
            up,
            low,
            temps: temps.to_vec(),
            coll_rate,
        })
    }
}
