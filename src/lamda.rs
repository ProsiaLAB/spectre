// //! Readers and writers for the Leiden Atomic and Molecular Database (LAMDA)

// use std::collections::HashMap;
// use std::fs::File;
// use std::io::BufReader;
// use std::io::{BufRead, Read};
// use std::path::Path;

// use crate::errors::database::LAMDAError;

// #[derive(Debug)]
// pub struct CollRate {
//     pub temp: f64,
//     pub rate: f64,
// }

// #[derive(Debug)]
// pub struct Level {
//     /// ID of the level
//     pub id: usize,
//     /// Energy of the level in cm^-1
//     pub energy: f64,
//     ///  Statistical weight (degeneracy) of the level, g = (2J + 1) * symmetry_factor
//     pub weight: f64,
//     /// Total angular momentum quantum number of the level.
//     pub j: usize,
// }

// #[derive(Debug, Default)]
// pub struct LevelData {
//     pub ids: Vec<usize>,
//     pub energies: Vec<f64>,
//     pub weights: Vec<f64>,
//     pub qnums: Vec<usize>,
// }

// #[derive(Debug)]
// pub struct RadTransition {
//     /// ID of the transition
//     pub id: usize,
//     /// Upper level ID
//     pub up: usize,
//     /// Lower level ID
//     pub low: usize,
//     /// Einstein A coefficient (s^-1)
//     pub einst_a: f64,
//     /// Frequency of the transition (GHz)
//     pub freq: f64,
//     /// Energy of the transition in (K)
//     pub energy: f64,
// }

// #[derive(Debug, Default)]
// pub struct RadSet {
//     pub ids: Vec<usize>,
//     pub ups: Vec<usize>,
//     pub lows: Vec<usize>,
//     pub einst_as: Vec<f64>,
//     pub freqs: Vec<f64>,
//     pub energies: Vec<f64>,
// }

// #[derive(Debug)]
// pub struct ColliTransition {
//     pub partner_id: String,
//     pub id: usize,
//     pub up: usize,
//     pub low: usize,
//     pub temps: Vec<f64>,
//     pub coll_rate: CollRate,
// }

// #[derive(Debug)]
// pub struct CollSet {
//     pub ids: Vec<usize>,
//     pub ups: Vec<usize>,
//     pub lows: Vec<usize>,
//     pub temps: Vec<f64>,
//     pub coll_rates: Vec<Vec<f64>>,
// }

// #[derive(Debug, Default)]
// pub struct LAMDAData {
//     pub levels: LevelData,
//     pub radset: RadSet,
//     pub collsets: HashMap<String, CollSet>,
// }

// impl LAMDAData {
//     pub fn from_reader<R: Read>(reader: R) -> Result<Self, LAMDAError> {
//         let reader = BufReader::new(reader);
//         let mut lines = reader.lines();

//         lines
//             .nth(1)
//             .ok_or_else(|| LAMDAError::ParseError("Missing molecule header".into()))??;
//         lines
//             .nth(1)
//             .ok_or_else(|| LAMDAError::ParseError("Missing weight".into()))??;

//         let level_count: usize = lines
//             .nth(1)
//             .ok_or_else(|| LAMDAError::ParseError("Missing energy levels count".into()))??
//             .parse()?;

//         lines
//             .next()
//             .ok_or_else(|| LAMDAError::ParseError("Missing energy level header".into()))??;

//         let level_data = Self::parse_energy_levels(&mut lines, level_count)?;

//         let transition_count: usize = lines
//             .nth(1)
//             .ok_or_else(|| LAMDAError::ParseError("Missing radiative transition count".into()))??
//             .parse()?;

//         lines.next().ok_or_else(|| {
//             LAMDAError::ParseError("Missing radiative transition header".into())
//         })??;

//         let radset = Self::parse_rad_transitions(&mut lines, transition_count)?;

//         let colli_partner_count: usize = lines
//             .nth(1)
//             .ok_or_else(|| {
//                 LAMDAError::ParseError("Missing collisional transition partner count".into())
//             })??
//             .parse()?;

//         let collsets = Self::parse_colli_transitions(&mut lines, colli_partner_count)?;

//         let mut coll_transitions = HashMap::new();
//         for _ in 0..colli_partner_count {
//             let partner_info = lines.nth(1).ok_or_else(|| {
//                 LAMDAError::ParseError("Missing partner transition header".into())
//             })??;
//             let partner_id = partner_info
//                 .split_whitespace()
//                 .next()
//                 .ok_or_else(|| LAMDAError::ParseError("Missing partner transition ID".into()))?;
//             let partner_name = match partner_id {
//                 "1" => "H2",
//                 "2" => "p-H2",
//                 "3" => "o-H2",
//                 "4" => "e",
//                 "5" => "H",
//                 "6" => "He",
//                 "7" => "H+",
//                 _ => {
//                     return Err(LAMDAError::ParseError(
//                         "Invalid partner transition ID".into(),
//                     ));
//                 }
//             };
//             let colli_transitions_count: usize = lines
//                 .nth(1)
//                 .ok_or_else(|| {
//                     LAMDAError::ParseError("Missing collisional transition count".into())
//                 })??
//                 .parse()?;
//             let _: usize = lines
//                 .nth(1)
//                 .ok_or_else(|| {
//                     LAMDAError::ParseError("Missing collisional transition header".into())
//                 })??
//                 .parse()?;
//             let colli_transitions_temps = lines
//                 .nth(1)
//                 .ok_or_else(|| {
//                     LAMDAError::ParseError("Missing collisional transition temperatures".into())
//                 })??
//                 .split_whitespace()
//                 .map(|x| {
//                     x.parse::<f64>()
//                         .map_err(|e| LAMDAError::ParseError(e.to_string()))
//                 })
//                 .collect::<Result<Vec<_>, _>>()?;

//             lines.next().ok_or_else(|| {
//                 LAMDAError::ParseError("Missing collisional transition data".into())
//             })??;
//             let colli_transitions = (0..colli_transitions_count)
//                 .map(|_| {
//                     Self::parse_colli_transition(
//                         &mut lines,
//                         &colli_transitions_temps,
//                         colli_transitions_count,
//                     )
//                 })
//                 .collect::<Result<Vec<_>, _>>()?;

//             coll_transitions.insert(partner_name.to_string(), colli_transitions);
//         }

//         Ok(Self {
//             levels,
//             radset: rad_transitions,
//             collsets: coll_transitions,
//         })
//     }

//     pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LAMDAError> {
//         let file = File::open(path)?;
//         let reader = BufReader::new(file);
//         Self::from_reader(reader)
//     }

//     fn parse_energy_levels<I>(lines: &mut I, level_count: usize) -> Result<LevelData, LAMDAError>
//     where
//         I: Iterator<Item = Result<String, std::io::Error>>,
//     {
//         let mut iter = lines
//             .filter_map(|line_result| match line_result {
//                 Ok(line) if !line.starts_with('!') => Some(line),
//                 _ => None,
//             })
//             .map(|level| {
//                 let mut fields = level.split_whitespace();
//                 let id: usize = fields
//                     .next()
//                     .expect("Failed to parse level ID")
//                     .parse()
//                     .expect("Failed to parse level ID");
//                 let energy: f64 = fields
//                     .next()
//                     .expect("Failed to parse level energy")
//                     .parse()
//                     .expect("Failed to parse level energy");
//                 let weight: f64 = fields
//                     .next()
//                     .expect("Failed to parse level statistical weight")
//                     .parse()
//                     .expect("Failed to parse level statistical weight");
//                 let qnum: usize = fields
//                     .next()
//                     .expect("Failed to parse level J")
//                     .parse()
//                     .expect("Failed to parse level J");
//                 (id, energy, weight, qnum)
//             });

//         let mut ids = vec![0; level_count];
//         let mut energies = vec![0.0; level_count];
//         let mut weights = vec![0.0; level_count];
//         let mut qnums = vec![0; level_count];

//         for (((id, energy), weight), qnum) in ids
//             .iter_mut()
//             .zip(energies.iter_mut())
//             .zip(weights.iter_mut())
//             .zip(qnums.iter_mut())
//         {
//             let (id_val, energy_val, weight_val, qnum_val) =
//                 iter.next().expect("Failed to parse level");
//             *id = id_val;
//             *energy = energy_val;
//             *weight = weight_val;
//             *qnum = qnum_val;
//         }

//         Ok(LevelData {
//             ids,
//             energies,
//             weights,
//             qnums,
//         })
//     }

//     fn parse_rad_transitions<I>(
//         lines: &mut I,
//         transition_count: usize,
//     ) -> Result<RadSet, LAMDAError>
//     where
//         I: Iterator<Item = Result<String, std::io::Error>>,
//     {
//         let mut iter = lines
//             .filter_map(|line_result| match line_result {
//                 Ok(line) if !line.starts_with('!') => Some(line),
//                 _ => None,
//             })
//             .map(|transition| {
//                 let mut transition_fields = transition.split_whitespace();
//                 let trans_id: usize = transition_fields
//                     .next()
//                     .expect("Failed to parse level ID")
//                     .parse()
//                     .expect("Failed to parse level ID");
//                 let upper_level: usize = transition_fields
//                     .next()
//                     .expect("Failed to parse level energy")
//                     .parse()
//                     .expect("Failed to parse level energy");
//                 let lower_level: usize = transition_fields
//                     .next()
//                     .expect("Failed to parse level statistical weight")
//                     .parse()
//                     .expect("Failed to parse level statistical weight");
//                 let einst_a: f64 = transition_fields
//                     .next()
//                     .expect("Failed to parse level J")
//                     .parse()
//                     .expect("Failed to parse level J");
//                 let freq: f64 = transition_fields
//                     .next()
//                     .expect("Failed to parse level J")
//                     .parse()
//                     .expect("Failed to parse level J");
//                 let energy: f64 = transition_fields
//                     .next()
//                     .expect("Failed to parse level J")
//                     .parse()
//                     .expect("Failed to parse energy");
//                 (trans_id, upper_level, lower_level, einst_a, freq, energy)
//             });

//         let mut ids = vec![0; transition_count];
//         let mut ups = vec![0; transition_count];
//         let mut lows = vec![0; transition_count];
//         let mut einst_as = vec![0.0; transition_count];
//         let mut freqs = vec![0.0; transition_count];
//         let mut energies = vec![0.0; transition_count];

//         for (((((id, upper_level), lower_level), einst_a), freq), energy) in ids
//             .iter_mut()
//             .zip(ups.iter_mut())
//             .zip(lows.iter_mut())
//             .zip(einst_as.iter_mut())
//             .zip(freqs.iter_mut())
//             .zip(energies.iter_mut())
//         {
//             let (id_val, up_val, low_val, einst_a_val, freq_val, energy_val) =
//                 iter.next().expect("Failed to parse transition");
//             *id = id_val;
//             *upper_level = up_val;
//             *lower_level = low_val;
//             *einst_a = einst_a_val;
//             *freq = freq_val;
//             *energy = energy_val;
//         }

//         Ok(RadSet {
//             ids,
//             ups,
//             lows,
//             einst_as,
//             freqs,
//             energies,
//         })
//     }

//     fn parse_colli_transitions<I>(
//         lines: &mut I,
//         partner_count: usize,
//     ) -> Result<HashMap<String, CollSet>, LAMDAError>
//     where
//         I: Iterator<Item = Result<String, std::io::Error>>,
//     {
//         let line = lines
//             .next()
//             .ok_or_else(|| LAMDAError::ParseError("Missing collisional transition".into()))??;

//         let collsets = HashMap::new();

//         for _ in 0..partner_count {
//             let partner_info = lines.nth(1).ok_or_else(|| {
//                 LAMDAError::ParseError("Missing partner transition header".into())
//             })??;
//             let partner_id = partner_info
//                 .split_whitespace()
//                 .next()
//                 .ok_or_else(|| LAMDAError::ParseError("Missing partner transition ID".into()))?;
//             let partner_name = match partner_id {
//                 "1" => "H2".to_string(),
//                 "2" => "p-H2".to_string(),
//                 "3" => "o-H2".to_string(),
//                 "4" => "e".to_string(),
//                 "5" => "H".to_string(),
//                 "6" => "He".to_string(),
//                 "7" => "H+".to_string(),
//                 _ => {
//                     return Err(LAMDAError::ParseError(
//                         "Invalid partner transition ID".into(),
//                     ));
//                 }
//             };
//             let colli_transitions_count: usize = lines
//                 .nth(1)
//                 .ok_or_else(|| {
//                     LAMDAError::ParseError("Missing collisional transition count".into())
//                 })??
//                 .parse()?;
//             let _: usize = lines
//                 .nth(1)
//                 .ok_or_else(|| {
//                     LAMDAError::ParseError("Missing collisional transition header".into())
//                 })??
//                 .parse()?;
//             let colli_transitions_temps = lines
//                 .nth(1)
//                 .ok_or_else(|| {
//                     LAMDAError::ParseError("Missing collisional transition temperatures".into())
//                 })??
//                 .split_whitespace()
//                 .map(|x| {
//                     x.parse::<f64>()
//                         .map_err(|e| LAMDAError::ParseError(e.to_string()))
//                 })
//                 .collect::<Result<Vec<_>, _>>()?;
//             lines.next().ok_or_else(|| {
//                 LAMDAError::ParseError("Missing collisional transition data".into())
//             })??;
//         }

//         Ok(collsets)
//     }

//     fn parse_colli_transition_set<I>(
//         lines: &mut I,
//         colli_transitions_temps: &[f64],
//         colli_transitions_count: usize,
//     ) -> Result<CollSet, LAMDAError>
//     where
//         I: Iterator<Item = Result<String, std::io::Error>>,
//     {
//         let mut iter = lines
//             .filter_map(|line_result| match line_result {
//                 Ok(line) if !line.starts_with('!') => Some(line),
//                 _ => None,
//             })
//             .map(|transition| {
//                 let mut transition_fields = transition.split_whitespace();
//                 let trans_id: usize = transition_fields
//                     .next()
//                     .expect("Failed to parse level ID")
//                     .parse()
//                     .expect("Failed to parse level ID");
//                 let upper_level: usize = transition_fields
//                     .next()
//                     .expect("Failed to parse level energy")
//                     .parse()
//                     .expect("Failed to parse level energy");
//                 let lower_level: usize = transition_fields
//                     .next()
//                     .expect("Failed to parse level statistical weight")
//                     .parse()
//                     .expect("Failed to parse level statistical weight");
//                 let coll_rates: Vec<Vec<f64>> = transition_fields.next();
//             });
//     }
// }
