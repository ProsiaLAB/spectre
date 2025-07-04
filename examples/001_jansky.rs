use spectre::units;
use spectre::units::length::{foot, meter};
use spectre::units::spectral_flux_density::{jansky, millijansky}; // Import millijansky
use uom::fmt::DisplayStyle::Abbreviation;

fn main() {
    let l1 = units::f64::Length::new::<meter>(100.0);

    println!(
        "{} = {}",
        l1.into_format_args(meter, Abbreviation),
        l1.into_format_args(foot, Abbreviation)
    );

    // Add similar println for Jy and mJy
    let s2 = units::f64::SpectralFluxDensity::new::<jansky>(134.0);
    println!(
        "{} = {}",
        s2.into_format_args(jansky, Abbreviation),
        s2.into_format_args(millijansky, Abbreviation)
    );
}
