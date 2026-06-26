use std::{
    convert::TryFrom,
    fmt,
    num::{NonZeroU32, NonZeroU64},
};

use serde::{Deserialize, Serialize, de};

use super::{
    SATS_PER_BTC,
    cross_leverage::CrossLeverage,
    error::{CrossQuantityValidationError, QuantityValidationError},
    margin::Margin,
    price::Price,
    quantity::{OrderQuantity, QuantityLike},
};

const MAX_QUANTITY_AT_MIN_LEVERAGE: u32 = 15_000_000;
const MAX_QUANTITY_AT_MAX_LEVERAGE: u32 = 10_000_000;
const QUANTITY_ROUNDING_MULTIPLE: u32 = 5;

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
/// use lnm_sdk::rest::v3::models::{CrossLeverage, CrossQuantity};
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
pub struct CrossQuantity(u32);

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
    /// use lnm_sdk::rest::v3::models::CrossQuantity;
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
        let rounded = as_f64.round().max(0.0) as u32;
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
    /// use lnm_sdk::rest::v3::models::{CrossLeverage, CrossQuantity};
    ///
    /// let leverage = CrossLeverage::try_from(50).unwrap();
    /// assert_eq!(CrossQuantity::max(leverage).as_u64(), 12_525_255);
    /// ```
    pub fn max(leverage: CrossLeverage) -> Self {
        let leverage_range = (CrossLeverage::MAX.as_u64() - CrossLeverage::MIN.as_u64()) as u32;
        let quantity_delta = MAX_QUANTITY_AT_MIN_LEVERAGE - MAX_QUANTITY_AT_MAX_LEVERAGE;
        let leverage_offset = (leverage.as_u64() - CrossLeverage::MIN.as_u64()) as u32;

        let numerator =
            MAX_QUANTITY_AT_MIN_LEVERAGE * leverage_range - leverage_offset * quantity_delta;
        let denominator = leverage_range * QUANTITY_ROUNDING_MULTIPLE;
        let quantity = numerator.div_ceil(denominator) * QUANTITY_ROUNDING_MULTIPLE;

        Self(quantity)
    }

    /// Returns the cross quantity value as its underlying `u32` representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::CrossQuantity;
    ///
    /// let quantity = CrossQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_u32(), 1_000);
    /// ```
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Returns the cross quantity value as a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::CrossQuantity;
    ///
    /// let quantity = CrossQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_u64(), 1_000);
    /// ```
    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }

    /// Returns the cross quantity value as an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::CrossQuantity;
    ///
    /// let quantity = CrossQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_i64(), 1_000);
    /// ```
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }

    /// Returns the cross quantity value as a `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::CrossQuantity;
    ///
    /// let quantity = CrossQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_f64(), 1_000.0);
    /// ```
    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    /// Adds a signed integer amount to this cross quantity and validates the result.
    ///
    /// The operand may be any primitive integer type, including negative values. Only the final
    /// result is validated against [`CrossQuantity::MIN`] and [`CrossQuantity::HARD_MAX`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::CrossQuantity;
    ///
    /// let base = CrossQuantity::try_from(1_000).unwrap();
    ///
    /// // Add another `CrossQuantity`
    /// let added = CrossQuantity::try_from(500).unwrap();
    /// let total = base.try_add(added).unwrap();
    /// assert_eq!(total.as_u32(), 1_500);
    ///
    /// // Add a raw integer
    /// let total = base.try_add(500u32).unwrap();
    /// assert_eq!(total.as_u32(), 1_500);
    ///
    /// // Subtract via a negative operand
    /// let total = base.try_add(-500i64).unwrap();
    /// assert_eq!(total.as_u32(), 500);
    /// ```
    pub fn try_add(self, other: impl TryInto<i128>) -> Result<Self, CrossQuantityValidationError> {
        let other = other
            .try_into()
            .map_err(|_| CrossQuantityValidationError::TooHigh { value: u128::MAX })?;
        let sum = i128::from(self)
            .checked_add(other)
            .ok_or(CrossQuantityValidationError::TooHigh { value: u128::MAX })?;

        Self::try_from(sum)
    }

    /// Subtracts a signed integer amount from this cross quantity and validates the result.
    ///
    /// The operand may be any primitive integer type, including negative values. Only the final
    /// result is validated against [`CrossQuantity::MIN`] and [`CrossQuantity::HARD_MAX`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::CrossQuantity;
    ///
    /// let base = CrossQuantity::try_from(1_000).unwrap();
    ///
    /// // Subtract another `CrossQuantity`
    /// let removed = CrossQuantity::try_from(500).unwrap();
    /// let remaining = base.try_sub(removed).unwrap();
    /// assert_eq!(remaining.as_u32(), 500);
    ///
    /// // Subtract a raw integer
    /// let remaining = base.try_sub(500u32).unwrap();
    /// assert_eq!(remaining.as_u32(), 500);
    ///
    /// // Add via a negative operand
    /// let remaining = base.try_sub(-500i64).unwrap();
    /// assert_eq!(remaining.as_u32(), 1_500);
    /// ```
    pub fn try_sub(self, other: impl TryInto<i128>) -> Result<Self, CrossQuantityValidationError> {
        let other = other
            .try_into()
            .map_err(|_| CrossQuantityValidationError::TooLow { value: i128::MIN })?;
        let difference = i128::from(self)
            .checked_sub(other)
            .ok_or(CrossQuantityValidationError::TooHigh { value: u128::MAX })?;

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
    /// use lnm_sdk::rest::v3::models::{CrossLeverage, CrossQuantity, Margin, Price};
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

        Self::try_from(qtd.floor() as u128)
    }
}

impl crate::sealed::Sealed for CrossQuantity {}

impl QuantityLike for CrossQuantity {
    fn as_f64(&self) -> f64 {
        self.as_f64()
    }
}

impl From<OrderQuantity> for CrossQuantity {
    fn from(value: OrderQuantity) -> Self {
        // OrderQuantity::MAX is less than CrossQuantity::HARD_MAX.
        Self::try_from(value.as_u32()).expect("must be valid")
    }
}

impl From<CrossQuantity> for u32 {
    fn from(value: CrossQuantity) -> Self {
        value.0
    }
}

impl From<CrossQuantity> for u64 {
    fn from(value: CrossQuantity) -> Self {
        value.0 as u64
    }
}

impl From<CrossQuantity> for u128 {
    fn from(value: CrossQuantity) -> Self {
        value.0 as u128
    }
}

impl From<CrossQuantity> for i64 {
    fn from(value: CrossQuantity) -> Self {
        value.0 as i64
    }
}

impl From<CrossQuantity> for i128 {
    fn from(value: CrossQuantity) -> Self {
        value.0 as i128
    }
}

impl From<CrossQuantity> for f64 {
    fn from(value: CrossQuantity) -> Self {
        value.0 as f64
    }
}

impl From<CrossQuantity> for NonZeroU32 {
    fn from(value: CrossQuantity) -> Self {
        NonZeroU32::new(value.0).expect("must be non-zero")
    }
}

impl From<CrossQuantity> for NonZeroU64 {
    fn from(value: CrossQuantity) -> Self {
        NonZeroU64::new(value.as_u64()).expect("must be non-zero")
    }
}

impl TryFrom<u8> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u16> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u32> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u64> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u128> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value < Self::MIN.0 as u128 {
            return Err(CrossQuantityValidationError::TooLow {
                value: value as i128,
            });
        }

        if value > Self::HARD_MAX.0 as u128 {
            return Err(CrossQuantityValidationError::TooHigh { value });
        }

        Ok(CrossQuantity(value as u32))
    }
}

impl TryFrom<usize> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<i8> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i16> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i32> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(quantity: i32) -> Result<Self, Self::Error> {
        Self::try_from(quantity as i128)
    }
}

impl TryFrom<i64> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i128> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: i128) -> Result<Self, Self::Error> {
        if value < Self::MIN.0 as i128 {
            return Err(CrossQuantityValidationError::TooLow { value });
        }

        if value > Self::HARD_MAX.0 as i128 {
            return Err(CrossQuantityValidationError::TooHigh {
                value: value as u128,
            });
        }

        Ok(CrossQuantity(value as u32))
    }
}

impl TryFrom<isize> for CrossQuantity {
    type Error = CrossQuantityValidationError;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
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

        Self::try_from(value as i128)
    }
}

impl TryInto<OrderQuantity> for CrossQuantity {
    type Error = QuantityValidationError;

    fn try_into(self) -> Result<OrderQuantity, Self::Error> {
        OrderQuantity::try_from(self.as_u64())
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
        serializer.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for CrossQuantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let quantity = u32::deserialize(deserializer)?;
        CrossQuantity::try_from(quantity).map_err(|e| de::Error::custom(e.to_string()))
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
            CrossQuantityValidationError::TooHigh { value }
                if value == (CrossQuantity::HARD_MAX.as_u64() + CrossQuantity::MIN.as_u64()) as u128
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
            CrossQuantityValidationError::TooLow { value } if value == -1
        ));
    }

    #[test]
    fn test_try_add_cross_quantity_with_primitive_integer() {
        let base = CrossQuantity::try_from(1_000).unwrap();

        let total = base.try_add(500u32).unwrap();
        assert_eq!(total, CrossQuantity::try_from(1_500).unwrap());

        let total = base.try_add(500usize).unwrap();
        assert_eq!(total, CrossQuantity::try_from(1_500).unwrap());

        let total = base.try_add(500u128).unwrap();
        assert_eq!(total, CrossQuantity::try_from(1_500).unwrap());
    }

    #[test]
    fn test_try_sub_cross_quantity_with_primitive_integer() {
        let base = CrossQuantity::try_from(1_000).unwrap();

        let remaining = base.try_sub(500u32).unwrap();
        assert_eq!(remaining, CrossQuantity::try_from(500).unwrap());

        let remaining = base.try_sub(500usize).unwrap();
        assert_eq!(remaining, CrossQuantity::try_from(500).unwrap());

        let remaining = base.try_sub(500u128).unwrap();
        assert_eq!(remaining, CrossQuantity::try_from(500).unwrap());
    }

    #[test]
    fn test_try_add_cross_quantity_negative_operand_reduces_value() {
        let base = CrossQuantity::try_from(1_000).unwrap();

        let total = base.try_add(-500i64).unwrap();
        assert_eq!(total, CrossQuantity::try_from(500).unwrap());
    }

    #[test]
    fn test_try_add_cross_quantity_negative_operand_fails_when_below_min() {
        let error = CrossQuantity::try_from(500)
            .unwrap()
            .try_add(-501i64)
            .err()
            .unwrap();

        assert!(matches!(
            error,
            CrossQuantityValidationError::TooLow { value } if value == -1
        ));
    }

    #[test]
    fn test_try_sub_cross_quantity_negative_operand_increases_value() {
        let base = CrossQuantity::try_from(1_000).unwrap();

        let total = base.try_sub(-500i64).unwrap();
        assert_eq!(total, CrossQuantity::try_from(1_500).unwrap());
    }

    #[test]
    fn test_try_sub_cross_quantity_large_negative_operand_reports_too_high() {
        let error = CrossQuantity::HARD_MAX.try_sub(i128::MIN).err().unwrap();

        assert!(matches!(
            error,
            CrossQuantityValidationError::TooHigh { value } if value == u128::MAX
        ));
    }

    #[test]
    fn test_try_add_cross_quantity_overflow_reports_true_value() {
        let error = CrossQuantity::HARD_MAX.try_add(10u32).err().unwrap();

        assert!(matches!(
            error,
            CrossQuantityValidationError::TooHigh { value }
                if value == CrossQuantity::HARD_MAX.as_u64() as u128 + 10
        ));
    }

    #[test]
    fn test_cross_quantity_to_non_zero_u64() {
        let non_zero: NonZeroU64 = CrossQuantity::try_from(1_000).unwrap().into();
        assert_eq!(non_zero.get(), 1_000);
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
