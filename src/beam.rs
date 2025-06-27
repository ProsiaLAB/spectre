//! Implementation of `radio-beam` Python package in Rust

use std::f64::consts::PI;

use uom::si::angle::{degree, radian, second};
use uom::si::f64::{Angle, SolidAngle};
use uom::si::solid_angle::steradian;

use crate::constants::{FWHM_TO_AREA, SIGMA_TO_FWHM};
use crate::errors::radio::BeamError;

#[derive(Debug)]
pub struct Beam {
    /// The FWHM major axis
    pub major: Angle,
    /// The FWHM minor axis
    pub minor: Angle,
    /// The beam position angle
    pub pa: Angle,
    /// The area of the beam.  This is an alternative to specifying the
    /// major/minor/PA, and will create those values assuming a circular
    /// Gaussian beam.
    pub area: SolidAngle,
}

impl Beam {
    pub fn new(
        major: Option<Angle>,
        minor: Option<Angle>,
        pa: Option<Angle>,
        area: Option<SolidAngle>,
    ) -> Result<Self, BeamError> {
        let (major, minor, pa) = if let Some(area_val) = area {
            if major.is_some() || minor.is_some() || pa.is_some() {
                return Err(BeamError::ExclusiveParameterConflict);
            }
            let rad = (area_val.get::<steradian>() / (2.0 * PI)).sqrt();
            let fwhm = rad * SIGMA_TO_FWHM;
            (
                Angle::new::<second>(fwhm),
                Angle::new::<second>(fwhm),
                Angle::new::<degree>(0.0),
            )
        } else {
            let major_val = match major {
                Some(m) => m,
                None => return Err(BeamError::MissingParameter),
            };
            let minor_val = minor.unwrap_or(major_val);
            let pa_val = pa.unwrap_or(Angle::new::<degree>(0.0));
            if minor_val > major_val {
                return Err(BeamError::MinorGreaterThanMajor);
            }
            (major_val, minor_val, pa_val)
        };

        let computed_area = Self::to_area(major, minor);
        Ok(Self {
            major,
            minor,
            pa,
            area: computed_area,
        })
    }

    fn to_area(major: Angle, minor: Angle) -> SolidAngle {
        SolidAngle::new::<steradian>(major.get::<radian>() * minor.get::<radian>() * FWHM_TO_AREA)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::approx_eq;

    use uom::si::angle::{degree, second};
    use uom::si::f64::{Angle, SolidAngle};
    use uom::si::solid_angle::steradian;

    const TOLERANCE: f64 = 1e-14;

    #[test]
    fn test_beam_from_axes() {
        let major = Angle::new::<second>(5.0);
        let minor = Angle::new::<second>(3.0);
        let pa = Angle::new::<degree>(45.0);

        let beam = Beam::new(Some(major), Some(minor), Some(pa), None).unwrap();
        assert_eq!(beam.major.get::<second>(), 5.0);
        assert!(approx_eq(beam.minor.get::<second>(), 3.0, TOLERANCE));
        assert_eq!(beam.pa.get::<degree>(), 45.0);
        let expected_area = major.get::<radian>() * minor.get::<radian>() * FWHM_TO_AREA;
        assert!(approx_eq(
            beam.area.get::<steradian>(),
            expected_area,
            1e-14
        ));
    }

    #[test]
    fn test_beam_default_minor_and_pa() {
        let major = Angle::new::<second>(2.5);

        let beam = Beam::new(Some(major), None, None, None).unwrap();
        assert_eq!(beam.major.get::<second>(), 2.5);
        assert_eq!(beam.minor.get::<second>(), 2.5);
        assert_eq!(beam.pa.get::<degree>(), 0.0);
    }

    #[test]
    fn test_beam_from_area() {
        // Area for circular beam with FWHM of 4 arcsec
        let fwhm_rad = Angle::new::<second>(4.0).get::<radian>();
        dbg!(fwhm_rad);
        let sigma = fwhm_rad / SIGMA_TO_FWHM;
        dbg!(sigma);
        let area_val = 2.0 * PI * sigma * sigma;
        dbg!(area_val);
        let area = SolidAngle::new::<steradian>(area_val);
        dbg!(area);

        let beam = Beam::new(None, None, None, Some(area)).unwrap();
        let computed = beam.major.get::<radian>();
        let expected = fwhm_rad;
        println!("computed: {}", computed);
        println!("expected: {}", expected);
        println!("diff: {}", (computed - expected).abs());

        assert!((computed - expected).abs() < TOLERANCE);
        assert_eq!(beam.major.get::<second>(), beam.minor.get::<second>());
        assert_eq!(beam.pa.get::<degree>(), 0.0);
    }

    #[test]
    fn test_minor_greater_than_major_error() {
        let major = Angle::new::<second>(2.0);
        let minor = Angle::new::<second>(3.0);
        let err = Beam::new(Some(major), Some(minor), None, None).unwrap_err();
        matches!(err, BeamError::MinorGreaterThanMajor);
    }

    #[test]
    fn test_missing_all_params_error() {
        let err = Beam::new(None, None, None, None).unwrap_err();
        matches!(err, BeamError::MissingParameter);
    }

    #[test]
    fn test_conflicting_params_error() {
        let major = Angle::new::<second>(2.0);
        let area = SolidAngle::new::<steradian>(1e-8);
        let err = Beam::new(Some(major), None, None, Some(area)).unwrap_err();
        matches!(err, BeamError::ExclusiveParameterConflict);
    }
}
