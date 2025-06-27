use std::f64::consts::{LN_2, PI};

/// Convert 2-dimensional Gaussian FWHM^2 to effective area.
pub const FWHM_TO_AREA: f64 = 2.0 * PI / (8.0 * LN_2);

/// Convert 2-dimensional Gaussian sigma^2 to FWHM
/// == sqrt(8*ln(2))
pub const SIGMA_TO_FWHM: f64 = 2.35482004503;
