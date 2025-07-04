use std::ops::Sub;

use uom::si::f64::Angle;

pub trait ApproxEq: Copy + PartialOrd + Sub<Output = Self> {
    fn abs_diff(self, other: Self) -> Self;
    fn approx_eq(self, other: Self, tolerance: Self) -> bool {
        self.abs_diff(other) < tolerance
    }
}

impl ApproxEq for f64 {
    fn abs_diff(self, other: Self) -> Self {
        (self - other).abs()
    }
}

impl ApproxEq for Angle {
    fn abs_diff(self, other: Self) -> Self {
        (self - other).abs()
    }
}

pub fn approx_eq<T: ApproxEq>(a: T, b: T, tolerance: T) -> bool {
    a.approx_eq(b, tolerance)
}
