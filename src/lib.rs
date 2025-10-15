//! `spectre` is intended to implement a subset of functionalities from `astropy`,
//! `astroquery`, and `specutils` libraries. And perhaps some more for radio/sub-mm
//! astronomy.
//!
//! Current plan is to implement the following:
//!
//! * `astroquery.linelists.cdms` - We will only implement readers, and not a full-fledged query tool.
//! * `astroquery.lamda` - We will only implement readers, and not a full-fledged query tool.
//! * `astroquery.hitran` - We will only implement readers, and not a full-fledged query tool.
//! * `astroquery.jplspec` - We will only implement readers, and not a full-fledged query tool.
//!
//! * `specutils` - Likely full-fledged implementation.
//! * `spectral_cube` - Likely full-fledged implementation.
//! * `radio_beam` - Likely full-fledged implementation.

// pub mod beam;
pub mod cdms;
pub mod constants;
pub mod errors;
pub mod hitran;
pub mod io;
pub mod jpl;
pub mod lamda;
// pub mod utils;
