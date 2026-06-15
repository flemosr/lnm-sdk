use std::{convert::TryFrom, fmt, num::NonZeroU64};

use serde::{Deserialize, Serialize, de};

use crate::shared::models::{
    SATS_PER_BTC, margin::Margin, price::Price, quantity::OrderQuantity, trade::TradeQuantity,
};

use super::{cross_leverage::CrossLeverage, error::CrossQuantityValidationError};

const MAX_QUANTITY_AT_MIN_LEVERAGE: u64 = 15_000_000;
const MAX_QUANTITY_AT_MAX_LEVERAGE: u64 = 10_000_000;
const QUANTITY_ROUNDING_MULTIPLE: u64 = 5;

/// A validated quantity value denominated in USD for futures cross positions.
///
/// Cross quantity represents the notional value of a cross-margin position in USD. This type
/// ensures quantity values are within the hard bounds accepted by cross-margin futures positions.
///
/// Cross quantity values must be:
/// + Integer values (whole USD amounts)
/// + Greater than or equal to [`CrossQuantity::MIN`] (1 USD)
/// + Less than or equal to [`CrossQuantity::HARD_MAX`] (15,000,000 USD)
///
/// # Examples
///
/// ```
/// use lnm_sdk::api_v3::models::{CrossLeverage, CrossQuantity};
///
/// // Create a cross quantity value from a USD amount
/// let quantity = CrossQuantity::try_from(1_000).unwrap();
/// assert_eq!(quantity.as_u64(), 1_000);
///
/// // Get the leverage-specific maximum quantity
/// let leverage = CrossLeverage::try_from(100).unwrap();
/// assert_eq!(CrossQuantity::max(leverage).as_u64(), 10_000_000);
///
/// // Values outside the hard valid range will fail
/// assert!(CrossQuantity::try_from(0).is_err());
/// assert!(CrossQuantity::try_from(16_000_000).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrossQuantity(u64);

impl CrossQuantity {
    /// The minimum allowed cross quantity value (1 USD).
    pub const MIN: Self = Self(1);

    /// The hard maximum allowed cross quantity value (15,000,000 USD).
    pub const HARD_MAX: Self = Self(MAX_QUANTITY_AT_MIN_LEVERAGE);

    /// Creates a `CrossQuantity` by rounding and bounding the given value to the hard valid range.
    ///
    /// This method rounds the input to the nearest integer and bounds it to the range
    /// ([CrossQuantity::MIN], [CrossQuantity::HARD_MAX]). It should be used to ensure a valid
    /// `CrossQuantity` without error handling.
    ///
    /// **Note:** In order to check whether a value is a valid cross quantity and receive an error
    /// for invalid values, use [`CrossQuantity::try_from`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::CrossQuantity;
    ///
    /// // Values within range are rounded
    /// let q = CrossQuantity::bounded(1_234.7);
    /// assert_eq!(q.as_u64(), 1_235);
    ///
    /// // Values below minimum are bounded to MIN
    /// let q = CrossQuantity::bounded(-1);
    /// assert_eq!(q, CrossQuantity::MIN);
    ///
    /// // Values above the hard maximum are bounded to HARD_MAX
    /// let q = CrossQuantity::bounded(16_000_000);
    /// assert_eq!(q, CrossQuantity::HARD_MAX);
    /// ```
    pub fn bounded<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        let as_f64: f64 = value.into();
        let rounded = as_f64.round().max(0.0) as u64;
        let clamped = rounded.clamp(Self::MIN.0, Self::HARD_MAX.0);

        Self(clamped)
    }

    /// Returns the leverage-specific maximum cross quantity.
    ///
    /// The maximum follows the LN Markets cross-position limit curve from 1x to 100x and rounds up
    /// to the next 5 USD increment.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::{CrossLeverage, CrossQuantity};
    ///
    /// let leverage = CrossLeverage::try_from(50).unwrap();
    /// assert_eq!(CrossQuantity::max(leverage).as_u64(), 12_525_255);
    /// ```
    pub fn max(leverage: CrossLeverage) -> Self {
        let leverage_range = CrossLeverage::MAX.as_u64() - CrossLeverage::MIN.as_u64();
        let quantity_delta = MAX_QUANTITY_AT_MIN_LEVERAGE - MAX_QUANTITY_AT_MAX_LEVERAGE;
        let leverage_offset = leverage.as_u64() - CrossLeverage::MIN.as_u64();

        let numerator =
            MAX_QUANTITY_AT_MIN_LEVERAGE * leverage_range - leverage_offset * quantity_delta;
        let denominator = leverage_range * QUANTITY_ROUNDING_MULTIPLE;
        let quantity = numerator.div_ceil(denominator) * QUANTITY_ROUNDING_MULTIPLE;

        Self(quantity)
    }

    /// Returns the cross quantity value as its underlying `u64` representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::CrossQuantity;
    ///
    /// let quantity = CrossQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_u64(), 1_000);
    /// ```
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Returns the cross quantity value as a `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::CrossQuantity;
    ///
    /// let quantity = CrossQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_f64(), 1_000.0);
    /// ```
    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    /// Adds two cross quantity values and validates the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::CrossQuantity;
    ///
    /// let base = CrossQuantity::try_from(1_000).unwrap();
    /// let added = CrossQuantity::try_from(500).unwrap();
    ///
    /// let total = base.try_add(added).unwrap();
    /// assert_eq!(total.as_u64(), 1_500);
    /// ```
    pub fn try_add(self, other: Self) -> Result<Self, CrossQuantityValidationError> {
        let sum = self.0.checked_add(other.0).unwrap_or(u64::MAX);

        Self::try_from(sum)
    }

    /// Subtracts a cross quantity value from another and validates the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::CrossQuantity;
    ///
    /// let base = CrossQuantity::try_from(1_000).unwrap();
    /// let removed = CrossQuantity::try_from(500).unwrap();
    ///
    /// let remaining = base.try_sub(removed).unwrap();
    /// assert_eq!(remaining.as_u64(), 500);
    /// ```
    pub fn try_sub(self, other: Self) -> Result<Self, CrossQuantityValidationError> {
        let difference = self.0.checked_sub(other.0).unwrap_or(0);

        Self::try_from(difference)
    }

    /// Calculates cross quantity (USD) from running margin (sats), price (BTC/USD), and leverage.
    ///
    /// The quantity is calculated using the formula:
    ///
    /// quantity = (running_margin * leverage * price) / SATS_PER_BTC
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::api_v3::models::{CrossLeverage, CrossQuantity, Margin, Price};
    ///
    /// let running_margin = Margin::try_from(10_000).unwrap(); // Margin in sats
    /// let price = Price::try_from(100_000.0).unwrap(); // Price in USD/BTC
    /// let leverage = CrossLeverage::try_from(10).unwrap();
    ///
    /// let quantity = CrossQuantity::try_calculate(running_margin, price, leverage).unwrap();
    ///
    /// assert_eq!(quantity.as_u64(), 100); // 100 [USD]
    /// ```
    pub fn try_calculate(
        running_margin: Margin,
        price: Price,
        leverage: CrossLeverage,
    ) -> Result<Self, CrossQuantityValidationError> {
        let qtd =
            running_margin.as_f64() * leverage.as_u64() as f64 * price.as_f64() / SATS_PER_BTC;

        Self::try_from(qtd.floor() as u64)
    }
}

impl TradeQuantity for CrossQuantity {
    fn as_f64(&self) -> f64 {
        self.as_f64()
    }
}

impl From<OrderQuantity> for CrossQuantity {
    fn from(value: OrderQuantity) -> Self {
        // OrderQuantity::MAX is less than CrossQuantity::HARD_MAX.
        Self::try_from(value.as_u64()).expect("must be valid")
    }
}

impl From<CrossQuantity> for u64 {
    fn from(value: CrossQuantity) -> Self {
        value.0
    }
}

impl From<CrossQuantity> for NonZeroU64 {
    fn from(value: CrossQuantity) -> Self {
        NonZeroU64::new(value.0).expect("must be non-zero")
    }
}

impl From<CrossQuantity> for f64 {
    fn from(value: CrossQuantity) -> Self {
        value.0 as f64
    }
}

impl TryFrom<u8> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u64)
    }
}

impl TryFrom<u16> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_from(value as u64)
    }
}

impl TryFrom<u32> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::try_from(value as u64)
    }
}

impl TryFrom<u64> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value < Self::MIN.0 {
            return Err(CrossQuantityValidationError::TooLow { value });
        }

        if value > Self::HARD_MAX.0 {
            return Err(CrossQuantityValidationError::TooHigh { value });
        }

        Ok(CrossQuantity(value))
    }
}

impl TryFrom<i8> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::try_from(value.max(0) as u64)
    }
}

impl TryFrom<i16> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::try_from(value.max(0) as u64)
    }
}

impl TryFrom<i32> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(quantity: i32) -> Result<Self, Self::Error> {
        Self::try_from(quantity.max(0) as u64)
    }
}

impl TryFrom<i64> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::try_from(value.max(0) as u64)
    }
}

impl TryFrom<usize> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(value as u64)
    }
}

impl TryFrom<isize> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Self::try_from(value.max(0) as u64)
    }
}

impl TryFrom<f32> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::try_from(value as f64)
    }
}

impl TryFrom<f64> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.fract() != 0.0 {
            return Err(CrossQuantityValidationError::NotAnInteger { value });
        }

        Self::try_from(value.max(0.) as u64)
    }
}

impl fmt::Display for CrossQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for CrossQuantity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for CrossQuantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let quantity_u64 = u64::deserialize(deserializer)?;
        CrossQuantity::try_from(quantity_u64).map_err(|e| de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_add_cross_quantity() {
        let base = CrossQuantity::try_from(1_000).unwrap();
        let added = CrossQuantity::try_from(500).unwrap();

        let total = base.try_add(added).unwrap();

        assert_eq!(total, CrossQuantity::try_from(1_500).unwrap());
    }

    #[test]
    fn test_try_add_cross_quantity_fails_above_hard_max() {
        let error = CrossQuantity::HARD_MAX
            .try_add(CrossQuantity::MIN)
            .err()
            .unwrap();

        assert!(matches!(
            error,
            CrossQuantityValidationError::TooHigh { value } if value == CrossQuantity::HARD_MAX.as_u64() + CrossQuantity::MIN.as_u64()
        ));
    }

    #[test]
    fn test_try_sub_cross_quantity() {
        let base = CrossQuantity::try_from(1_000).unwrap();
        let removed = CrossQuantity::try_from(500).unwrap();

        let remaining = base.try_sub(removed).unwrap();

        assert_eq!(remaining, CrossQuantity::try_from(500).unwrap());
    }

    #[test]
    fn test_try_sub_cross_quantity_fails_below_min() {
        let error = CrossQuantity::MIN
            .try_sub(CrossQuantity::MIN)
            .err()
            .unwrap();

        assert!(matches!(
            error,
            CrossQuantityValidationError::TooLow { value } if value == 0
        ));

        let error = CrossQuantity::MIN
            .try_sub(CrossQuantity::try_from(2).unwrap())
            .err()
            .unwrap();

        assert!(matches!(
            error,
            CrossQuantityValidationError::TooLow { value } if value == 0
        ));
    }

    #[test]
    fn test_max_cross_quantity() {
        let test_cases = [
            (100, 10_000_000),
            (80, 11_010_105),
            (50, 12_525_255),
            (20, 14_040_405),
            (10, 14_545_455),
            (3, 14_898_990),
            (2, 14_949_495),
            (1, 15_000_000),
        ];

        for (leverage, expected_quantity) in test_cases {
            let leverage = CrossLeverage::try_from(leverage).unwrap();

            assert_eq!(CrossQuantity::max(leverage).as_u64(), expected_quantity);
        }
    }

    #[test]
    fn test_calculate_cross_quantity() {
        let margin = Margin::try_from(1_000).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = CrossLeverage::try_from(1).unwrap();

        let quantity = CrossQuantity::try_calculate(margin, price, leverage).unwrap();
        assert_eq!(quantity, CrossQuantity::MIN);

        let margin = Margin::try_from(500_000_000).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = CrossLeverage::try_from(30).unwrap();

        let quantity = CrossQuantity::try_calculate(margin, price, leverage).unwrap();
        assert_eq!(quantity, CrossQuantity::HARD_MAX);

        let margin = Margin::try_from(9).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = CrossLeverage::try_from(1).unwrap();

        let quantity_validation_error = CrossQuantity::try_calculate(margin, price, leverage)
            .err()
            .unwrap();
        assert!(matches!(
            quantity_validation_error,
            CrossQuantityValidationError::TooLow { value: _ }
        ));

        let margin = Margin::try_from(500_100_000).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = CrossLeverage::try_from(30).unwrap();

        let quantity_validation_error = CrossQuantity::try_calculate(margin, price, leverage)
            .err()
            .unwrap();
        assert!(matches!(
            quantity_validation_error,
            CrossQuantityValidationError::TooHigh { value: _ }
        ));
    }
}
