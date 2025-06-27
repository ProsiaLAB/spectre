#[macro_use]
pub mod length {
    quantity! {
        /// Length (base unit meter, m).
        quantity: Length; "length";
        /// Length dimension, m.
        dimension: Q<
            P1,  // length
            Z0,  // mass
            Z0>; // time
        units {
            @meter: 1.0E0; "m", "meter", "meters";
            @foot: 3.048E-1; "ft", "foot", "feet";
        }
    }
}

#[macro_use]
mod mass {
    quantity! {
        /// Mass (base unit kilogram, kg).
        quantity: Mass; "mass";
        /// Mass dimension, kg.
        dimension: Q<
            Z0,  // length
            P1,  // mass
            Z0>; // time
        units {
            @kilogram: 1.0; "kg", "kilogram", "kilograms";
        }
    }
}

#[macro_use]
mod time {
    quantity! {
        /// Time (base unit second, s).
        quantity: Time; "time";
        /// Time dimension, s.
        dimension: Q<
            Z0,  // length
            Z0,  // mass
            P1>; // time
        units {
            @second: 1.0; "s", "second", "seconds";
        }
    }
}

#[macro_use]
pub mod spectral_flux_density {
    quantity! {
        /// Spectral Flux Density (base unit kg / s^2).
        /// Dimensions: Mass * Time^-2.
        quantity: SpectralFluxDensity; "spectral flux density";
        /// Spectral Flux Density dimension.
        dimension: Q<
            Z0,  // length (L^0)
            P1,  // mass (M^1)
            N2>; // time (T^-2)
        units {
            @jansky: 1.0E-26; "Jy", "jansky", "janskys";
            @millijansky: 1.0E-29; "mJy", "millijansky", "millijanskys";
        }
    }
}

system! {
    // Only list the *base* quantities here
    quantities: Q {
        length: meter, L;
        mass: kilogram, M;
        time: second, T;
    }

    units: U {
        // List all quantity modules, both base and derived
        mod length::Length,
        mod mass::Mass,
        mod time::Time,
        mod spectral_flux_density::SpectralFluxDensity,
    }
}

pub mod f32 {
    mod mks {
        pub use super::super::*;
    }

    Q!(self::mks, f32);
}

pub mod f64 {
    mod mks {
        pub use super::super::*;
    }

    Q!(self::mks, f64);
}
