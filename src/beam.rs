//! Implementation of `radio-beam` Python package in Rust
use std::cmp::PartialEq;
use std::f64::consts::PI;
use std::ops::{Div, Mul};

use crate::constants::{FWHM_TO_AREA, SIGMA_TO_FWHM};
use crate::errors::radio::BeamError;
use crate::utils::approx_eq;

#[derive(Debug)]
struct Angle {
    value: f64,
    unit: AngleUnit,
}

impl Angle {
    pub fn new<T>(value: f64) -> Self
    where
        T: Into<AngleUnit>,
    {
        Self {
            value,
            unit: T::into(),
        }
    }
}

#[derive(Debug)]
struct SolidAngle {
    value: f64,
    unit: SolidAngleUnit,
}

#[derive(Debug)]
enum AngleUnit {
    Radian,
    Degree,
    Arcsecond,
}

#[derive(Debug)]
enum SolidAngleUnit {
    Steradian,
}

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
        let (major, minor, pa) = if let Some(area) = area {
            if major.is_some() || minor.is_some() || pa.is_some() {
                return Err(BeamError::ExclusiveParameterConflict);
            }
            let rad = (area.value / (2.0 * PI)).sqrt();
            let fwhm_rad_val = rad * SIGMA_TO_FWHM;
            let fwhm_arcsec_val = Angle::new::<radian>(fwhm_rad_val).get::<arcsecond>();
            (
                Angle::new::<arcsecond>(fwhm_arcsec_val),
                Angle::new::<arcsecond>(fwhm_arcsec_val),
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

    pub fn convolve(self, other: Self) -> Self {
        let (new_major, new_minor, new_pa) = convolve(self, other);
        Beam::new(Some(new_major), Some(new_minor), Some(new_pa), None).unwrap()
    }

    pub fn deconvolve(self, other: Self) -> Self {
        let (new_major, new_minor, new_pa) = deconvolve(&self, &other);
        Beam::new(Some(new_major), Some(new_minor), Some(new_pa), None).unwrap()
    }

    pub fn is_circular(&self, rtol: Option<f64>) -> bool {
        let rtol = rtol.unwrap_or(1e-6);
        let frac_diff =
            (self.major.get::<degree>() - self.minor.get::<degree>()) / self.major.get::<degree>();
        frac_diff <= rtol
    }
}

impl Mul<Beam> for Beam {
    type Output = Beam;

    fn mul(self, other: Self) -> Self::Output {
        self.convolve(other)
    }
}

impl Div<Beam> for Beam {
    type Output = Beam;

    fn div(self, other: Self) -> Self::Output {
        self.deconvolve(other)
    }
}

impl PartialEq for Beam {
    fn eq(&self, other: &Self) -> bool {
        let atol_deg = 1e-10;
        let this_pa = self.pa.get::<degree>() % 180.0;
        let other_pa = other.pa.get::<degree>() % 180.0;

        let equal_pa = self.is_circular(None) || (this_pa - other_pa).abs() < atol_deg;

        let equal_major =
            (self.major.get::<degree>() - other.major.get::<degree>()).abs() < atol_deg;
        let equal_minor =
            (self.minor.get::<degree>() - other.minor.get::<degree>()).abs() < atol_deg;

        equal_major && equal_minor && equal_pa
    }
}

fn convolve(beam: Beam, other: Beam) -> (Angle, Angle, Angle) {
    // Unit is Angle^(-2)
    let alpha = (beam.major * beam.pa.cos()).powi(P2::new())
        + (beam.minor * beam.pa.sin()).powi(P2::new())
        + (other.major * other.pa.cos()).powi(P2::new())
        + (other.minor * other.pa.sin()).powi(P2::new());

    // Unit is Angle^(-2)
    let beta = (beam.major * beam.pa.sin()).powi(P2::new())
        + (beam.minor * beam.pa.cos()).powi(P2::new())
        + (other.major * other.pa.sin()).powi(P2::new())
        + (other.minor * other.pa.cos()).powi(P2::new());

    // Unit is Angle^(-2)
    let gamma = 2.0
        * ((beam.minor.powi(P2::new()) - beam.major.powi(P2::new()))
            * beam.pa.sin()
            * beam.pa.cos()
            + (other.minor.powi(P2::new()) - other.major.powi(P2::new()))
                * other.pa.sin()
                * other.pa.cos());

    let s = alpha + beta; // Unit is Angle^(-2)
    let t = ((alpha - beta).powi(P2::new()) + gamma.powi(P2::new())).sqrt(); // Unit is Angle^(-1)

    let new_major = (0.5 * (s + t)).sqrt(); // Unit is Angle^(-1)
    let new_minor = (0.5 * (s - t)).sqrt(); // Unit is Angle^(-1)

    let y = (-1.0 * gamma).value; // Unit is Angle^(-2)
    let x = (alpha - beta).value; // Unit is Angle^(-2)

    let new_par_radians = y.atan2(x);

    let tol_arcsec = Angle::new::<arcsecond>(1e-7); // 1 microarcsec of tolerance
    let pa_check = (gamma.abs() + (alpha - beta).abs()).sqrt();

    let new_pa = if approx_eq(pa_check.into(), Angle::new::<arcsecond>(0.0), tol_arcsec) {
        Angle::new::<degree>(0.0)
    } else {
        0.5 * Angle::new::<radian>(new_par_radians)
    };

    (new_major.into(), new_minor.into(), new_pa)
}

fn deconvolve(b1: &Beam, b2: &Beam) -> (Angle, Angle, Angle) {
    let alpha = (b1.major * b1.pa.cos()).powi(P2::new()) + (b1.minor * b1.pa.sin()).powi(P2::new())
        - (b2.major * b2.pa.cos()).powi(P2::new())
        - (b2.minor * b2.pa.sin()).powi(P2::new());

    let beta = (b1.major * b1.pa.sin()).powi(P2::new()) + (b1.minor * b1.pa.cos()).powi(P2::new())
        - (b2.major * b2.pa.sin()).powi(P2::new())
        - (b2.minor * b2.pa.cos()).powi(P2::new());

    let gamma = 2.0
        * ((b1.minor.powi(P2::new()) - b1.major.powi(P2::new())) * b1.pa.sin() * b1.pa.cos()
            - (b2.minor.powi(P2::new()) - b2.major.powi(P2::new())) * b2.pa.sin() * b2.pa.cos());

    let s = alpha + beta;
    let t = ((alpha - beta).powi(P2::new()) + gamma.powi(P2::new())).sqrt();

    let atol = f64::EPSILON;
    let atol_t = atol / 3600.0f64.powi(2);

    // To deconvolve, the beam must satisfy:
    // alpha < 0
    let alpha_cond = (alpha.value + atol) < 0.0;
    // beta < 0
    let beta_cond = (beta.value + atol) < 0.0;
    // s < t
    let st_cond = s.value < (t.value + atol_t);

    if alpha_cond || beta_cond || st_cond {
        (
            Angle::new::<radian>(0.0),
            Angle::new::<radian>(0.0),
            Angle::new::<radian>(0.0),
        )
    } else {
        let mut new_major = (0.5 * (s + t)).sqrt();
        let mut new_minor = (0.5 * (s - t)).sqrt();

        let y = (-1.0 * gamma).value; // Unit is Angle^(-2)
        let x = (alpha - beta).value; // Unit is Angle^(-2)

        let new_par_radians = y.atan2(x);

        let tol_arcsec = Angle::new::<arcsecond>(1e-7 / 3600.0); // 1 microarcsec of tolerance
        let pa_check = (gamma.abs() + (alpha - beta).abs()).sqrt();

        let new_pa = if approx_eq(pa_check.into(), Angle::new::<arcsecond>(0.0), tol_arcsec) {
            Angle::new::<degree>(0.0)
        } else {
            0.5 * Angle::new::<radian>(new_par_radians)
        };

        new_major.value += f64::EPSILON;
        new_minor.value += f64::EPSILON;

        (new_major.into(), new_minor.into(), new_pa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq; // For floating point comparisons

    // Helper function for creating angles in degrees
    fn deg(value: f64) -> Angle {
        Angle::new::<degree>(value)
    }

    // Helper function for creating solid angles in steradians
    fn sr(value: f64) -> SolidAngle {
        SolidAngle::new::<steradian>(value)
    }

    #[test]
    fn test_new_with_major_minor_pa() {
        let major = Some(deg(10.0));
        let minor = Some(deg(5.0));
        let pa = Some(deg(45.0));
        let beam = Beam::new(major, minor, pa, None).unwrap();

        assert_relative_eq!(beam.major.get::<degree>(), 10.0);
        assert_relative_eq!(beam.minor.get::<degree>(), 5.0);
        assert_relative_eq!(beam.pa.get::<degree>(), 45.0);

        // Expected area calculation: major_rad * minor_rad * FWHM_TO_AREA
        let expected_area = SolidAngle::new::<steradian>(
            deg(10.0).get::<radian>() * deg(5.0).get::<radian>() * FWHM_TO_AREA,
        );
        assert_relative_eq!(
            beam.area.get::<steradian>(),
            expected_area.get::<steradian>()
        );
    }

    #[test]
    fn test_new_with_only_major() {
        let major = Some(deg(10.0));
        let beam = Beam::new(major, None, None, None).unwrap();

        assert_relative_eq!(beam.major.get::<degree>(), 10.0);
        assert_relative_eq!(beam.minor.get::<degree>(), 10.0); // Minor should default to major
        assert_relative_eq!(beam.pa.get::<degree>(), 0.0); // PA should default to 0

        let expected_area = SolidAngle::new::<steradian>(
            deg(10.0).get::<radian>() * deg(10.0).get::<radian>() * FWHM_TO_AREA,
        );
        assert_relative_eq!(
            beam.area.get::<steradian>(),
            expected_area.get::<steradian>()
        );
    }

    #[test]
    fn test_new_with_area() {
        let test_area = sr(0.001); // Example area in steradians
        let beam = Beam::new(None, None, None, Some(test_area)).unwrap();

        // For area-defined beams, major and minor should be equal (circular)
        // and PA should be 0.
        assert_relative_eq!(beam.pa.get::<degree>(), 0.0);

        // Recalculate the expected FWHM from the area to verify major/minor
        let sigma_rad = (test_area.get::<steradian>() / (2.0 * PI)).sqrt();
        let expected_fwhm_rad = sigma_rad * SIGMA_TO_FWHM;

        assert_relative_eq!(beam.major.get::<radian>(), expected_fwhm_rad);
        assert_relative_eq!(beam.minor.get::<radian>(), expected_fwhm_rad);

        // The computed area should be very close to the input area
        assert_relative_eq!(
            beam.area.get::<steradian>(),
            test_area.get::<steradian>(),
            epsilon = 1e-6
        );
    }

    #[test]
    fn test_new_exclusive_parameter_conflict() {
        let major = Some(deg(10.0));
        let area = Some(sr(0.001));
        let error = Beam::new(major, None, None, area).unwrap_err();
        assert_eq!(error, BeamError::ExclusiveParameterConflict);

        let minor = Some(deg(5.0));
        let error = Beam::new(None, minor, None, area).unwrap_err();
        assert_eq!(error, BeamError::ExclusiveParameterConflict);

        let pa = Some(deg(30.0));
        let error = Beam::new(None, None, pa, area).unwrap_err();
        assert_eq!(error, BeamError::ExclusiveParameterConflict);
    }

    #[test]
    fn test_new_missing_parameter() {
        let error = Beam::new(None, Some(deg(5.0)), None, None).unwrap_err();
        assert_eq!(error, BeamError::MissingParameter);

        let error = Beam::new(None, None, Some(deg(45.0)), None).unwrap_err();
        assert_eq!(error, BeamError::MissingParameter);

        let error = Beam::new(None, None, None, None).unwrap_err();
        assert_eq!(error, BeamError::MissingParameter);
    }

    #[test]
    fn test_new_minor_greater_than_major() {
        let major = Some(deg(5.0));
        let minor = Some(deg(10.0));
        let error = Beam::new(major, minor, None, None).unwrap_err();
        assert_eq!(error, BeamError::MinorGreaterThanMajor);
    }

    #[test]
    fn test_to_area() {
        let major_angle = deg(10.0);
        let minor_angle = deg(5.0);

        // Expected area calculation: major_rad * minor_rad * FWHM_TO_AREA
        let expected_area_value =
            major_angle.get::<radian>() * minor_angle.get::<radian>() * FWHM_TO_AREA;
        let computed_area = Beam::to_area(major_angle, minor_angle);

        assert_relative_eq!(computed_area.get::<steradian>(), expected_area_value);

        // Test with circular beam
        let circular_major = deg(1.0);
        let circular_minor = deg(1.0);
        let expected_circular_area_value =
            circular_major.get::<radian>() * circular_minor.get::<radian>() * FWHM_TO_AREA;
        let computed_circular_area = Beam::to_area(circular_major, circular_minor);

        assert_relative_eq!(
            computed_circular_area.get::<steradian>(),
            expected_circular_area_value
        );

        // Test with zero values (should result in zero area)
        let zero_major = deg(0.0);
        let zero_minor = deg(0.0);
        let computed_zero_area = Beam::to_area(zero_major, zero_minor);
        assert_relative_eq!(computed_zero_area.get::<steradian>(), 0.0);
    }
}
