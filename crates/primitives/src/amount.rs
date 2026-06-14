//! TRX amount type.
//!
//! TRON denominates value in *sun*, where `1 TRX = 1_000_000 sun`. [`Trx`]
//! wraps an `i64` sun value to match the protobuf `sint64` representation.

use core::fmt;
use core::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

use crate::error::AmountError;

/// Number of sun in one TRX.
pub const SUN_PER_TRX: i64 = 1_000_000;

/// An amount of TRX, stored internally as `i64` sun.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Trx(i64);

impl Trx {
    /// Zero TRX.
    pub const ZERO: Trx = Trx(0);

    /// Construct directly from a sun value without validation.
    ///
    /// Negative values are representable here because malformed on-chain data
    /// must round-trip without panicking; prefer [`Trx::from_sun`] for
    /// user-facing input.
    pub const fn from_sun_unchecked(sun: i64) -> Self {
        Self(sun)
    }

    /// Construct from a sun value, rejecting negatives.
    pub fn from_sun(sun: i64) -> Result<Self, AmountError> {
        if sun < 0 {
            return Err(AmountError::Negative(sun));
        }
        Ok(Self(sun))
    }

    /// Construct from a floating-point TRX value (e.g. `1.5` TRX).
    ///
    /// Rejects negative and non-finite values, and values that overflow the
    /// `i64` sun range.
    pub fn from_trx(trx: f64) -> Result<Self, AmountError> {
        if !trx.is_finite() || trx < 0.0 {
            return Err(AmountError::OutOfRange(trx));
        }
        let sun = trx * SUN_PER_TRX as f64;
        if sun > i64::MAX as f64 {
            return Err(AmountError::OutOfRange(trx));
        }
        Ok(Self(sun.round() as i64))
    }

    /// The raw sun value.
    pub const fn as_sun(self) -> i64 {
        self.0
    }

    /// The value expressed as floating-point TRX (lossy for large amounts).
    pub fn as_trx(self) -> f64 {
        self.0 as f64 / SUN_PER_TRX as f64
    }

    /// Checked addition, returning `None` on `i64` overflow.
    pub fn checked_add(self, rhs: Trx) -> Option<Trx> {
        self.0.checked_add(rhs.0).map(Trx)
    }

    /// Checked subtraction, returning `None` on `i64` overflow.
    pub fn checked_sub(self, rhs: Trx) -> Option<Trx> {
        self.0.checked_sub(rhs.0).map(Trx)
    }
}

impl Add for Trx {
    type Output = Trx;
    fn add(self, rhs: Trx) -> Trx {
        Trx(self.0 + rhs.0)
    }
}

impl Sub for Trx {
    type Output = Trx;
    fn sub(self, rhs: Trx) -> Trx {
        Trx(self.0 - rhs.0)
    }
}

impl fmt::Display for Trx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} TRX", self.as_trx())
    }
}

impl fmt::Debug for Trx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Trx({} sun)", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        assert_eq!(Trx::from_trx(1.0).unwrap().as_sun(), 1_000_000);
        assert_eq!(Trx::from_trx(1.5).unwrap().as_sun(), 1_500_000);
        assert_eq!(Trx::from_sun(2_500_000).unwrap().as_trx(), 2.5);
    }

    #[test]
    fn rejects_negative() {
        assert!(Trx::from_sun(-1).is_err());
        assert!(Trx::from_trx(-1.0).is_err());
        assert!(Trx::from_trx(f64::NAN).is_err());
    }

    #[test]
    fn unchecked_allows_negative() {
        assert_eq!(Trx::from_sun_unchecked(-5).as_sun(), -5);
    }

    #[test]
    fn arithmetic() {
        let a = Trx::from_trx(1.0).unwrap();
        let b = Trx::from_trx(0.5).unwrap();
        assert_eq!((a + b).as_sun(), 1_500_000);
        assert_eq!((a - b).as_sun(), 500_000);
        assert_eq!(a.checked_add(b), Some(Trx::from_sun(1_500_000).unwrap()));
    }
}
