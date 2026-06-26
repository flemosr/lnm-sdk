use std::{convert::TryFrom, fmt};

use serde::{Deserialize, Serialize, de};

use crate::shared::models::{
    SATS_PER_BTC,
    error::QuantityValidationError,
    leverage::Leverage,
    margin::Margin,
    price::{PercentageCapped, Price},
    quantity::Quantity,
};

/// A validated quantity value denominated in USD.
///
/// `OrderQuantity` represents the notional value of a trading position in USD.
/// This type ensures that quantity values are within acceptable bounds and can be safely used when
/// trading futures.
///
/// `OrderQuantity` values must be:
/// + Integer values (whole USD amounts)
/// + Greater than or equal to [`OrderQuantity::MIN`] (1 USD)
/// + Less than or equal to [`OrderQuantity::MAX`] (500,000 USD)
///
/// # Examples
///
/// ```
/// use lnm_sdk::rest::v3::models::OrderQuantity;
///
/// // Create a quantity value from USD amount
/// let quantity = OrderQuantity::try_from(1_000).unwrap();
/// assert_eq!(quantity.as_u64(), 1_000);
///
/// // Values outside the valid range will fail
/// assert!(OrderQuantity::try_from(0).is_err());
/// assert!(OrderQuantity::try_from(600_000).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderQuantity(u32);

impl OrderQuantity {
    /// The minimum allowed quantity value (1 USD).
    pub const MIN: Self = Self(1);

    /// The maximum allowed quantity value (500,000 USD).
    pub const MAX: Self = Self(500_000);

    /// Creates a `OrderQuantity` by rounding and bounding the given value to the valid range.
    ///
    /// This method rounds the input to the nearest integer and bounds it to the range
    /// ([OrderQuantity::MIN], [OrderQuantity::MAX]).
    /// It should be used to ensure a valid `OrderQuantity` without error handling.
    ///
    /// **Note:** In order to check whether a value is a valid quantity and receive an error for
    /// invalid values, use [`OrderQuantity::try_from`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::OrderQuantity;
    ///
    /// // Values within range are rounded
    /// let q = OrderQuantity::bounded(1_234.7);
    /// assert_eq!(q.as_u64(), 1_235);
    ///
    /// // Values below minimum are bounded to MIN
    /// let q = OrderQuantity::bounded(-1);
    /// assert_eq!(q, OrderQuantity::MIN);
    ///
    /// // Values above maximum are bounded to MAX
    /// let q = OrderQuantity::bounded(600_000);
    /// assert_eq!(q, OrderQuantity::MAX);
    /// ```
    pub fn bounded<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        let as_f64: f64 = value.into();
        let rounded = as_f64.round().max(0.0) as u32;
        let clamped = rounded.clamp(Self::MIN.0, Self::MAX.0);

        Self(clamped)
    }

    /// Returns the quantity value as its underlying `u32` representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::OrderQuantity;
    ///
    /// let quantity = OrderQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_u32(), 1_000);
    /// ```
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Returns the quantity value as a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::OrderQuantity;
    ///
    /// let quantity = OrderQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_u64(), 1_000);
    /// ```
    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }

    /// Returns the quantity value as an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::OrderQuantity;
    ///
    /// let quantity = OrderQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_i64(), 1_000);
    /// ```
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }

    /// Returns the quantity value as a `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::OrderQuantity;
    ///
    /// let quantity = OrderQuantity::try_from(1_000).unwrap();
    /// assert_eq!(quantity.as_f64(), 1_000.0);
    /// ```
    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    /// Adds a signed integer amount to this quantity and validates the result.
    ///
    /// The operand may be any primitive integer type, including negative values. Only the final
    /// result is validated against [`OrderQuantity::MIN`] and [`OrderQuantity::MAX`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::OrderQuantity;
    ///
    /// let base = OrderQuantity::try_from(1_000).unwrap();
    ///
    /// // Add another `OrderQuantity`
    /// let added = OrderQuantity::try_from(500).unwrap();
    /// let total = base.try_add(added).unwrap();
    /// assert_eq!(total.as_u32(), 1_500);
    ///
    /// // Add a raw integer
    /// let total = base.try_add(500i32).unwrap();
    /// assert_eq!(total.as_u32(), 1_500);
    ///
    /// // Subtract via a negative operand
    /// let total = base.try_add(-500i64).unwrap();
    /// assert_eq!(total.as_u32(), 500);
    /// ```
    pub fn try_add(self, other: impl TryInto<i128>) -> Result<Self, QuantityValidationError> {
        let other = other
            .try_into()
            .map_err(|_| QuantityValidationError::TooHigh { value: u128::MAX })?;
        let sum = i128::from(self)
            .checked_add(other)
            .ok_or(QuantityValidationError::TooHigh { value: u128::MAX })?;

        Self::try_from(sum)
    }

    /// Subtracts a signed integer amount from this quantity and validates the result.
    ///
    /// The operand may be any primitive integer type, including negative values. Only the final
    /// result is validated against [`OrderQuantity::MIN`] and [`OrderQuantity::MAX`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::OrderQuantity;
    ///
    /// let base = OrderQuantity::try_from(1_000).unwrap();
    ///
    /// // Subtract another `OrderQuantity`
    /// let removed = OrderQuantity::try_from(500).unwrap();
    /// let remaining = base.try_sub(removed).unwrap();
    /// assert_eq!(remaining.as_u32(), 500);
    ///
    /// // Subtract a raw integer
    /// let remaining = base.try_sub(500i32).unwrap();
    /// assert_eq!(remaining.as_u32(), 500);
    ///
    /// // Add via a negative operand
    /// let remaining = base.try_sub(-500i64).unwrap();
    /// assert_eq!(remaining.as_u32(), 1_500);
    /// ```
    pub fn try_sub(self, other: impl TryInto<i128>) -> Result<Self, QuantityValidationError> {
        let other = other
            .try_into()
            .map_err(|_| QuantityValidationError::TooLow { value: i128::MIN })?;
        let difference = i128::from(self)
            .checked_sub(other)
            .ok_or(QuantityValidationError::TooHigh { value: u128::MAX })?;

        Self::try_from(difference)
    }

    /// Calculates quantity (USD) from margin (sats), price (BTC/USD), and leverage.
    ///
    /// The quantity is calculated using the formula:
    ///
    /// quantity = (margin * leverage * price) / SATS_PER_BTC
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::{OrderQuantity, Margin, Price, Leverage};
    ///
    /// let margin = Margin::try_from(10_000).unwrap(); // Margin in sats
    /// let price = Price::try_from(100_000.0).unwrap(); // Price in USD/BTC
    /// let leverage = Leverage::try_from(10.0).unwrap();
    ///
    /// let quantity = OrderQuantity::try_calculate(margin, price, leverage).unwrap();
    ///
    /// assert_eq!(quantity.as_u64(), 100); // 100 [USD]
    /// ```
    pub fn try_calculate(
        margin: Margin,
        price: Price,
        leverage: Leverage,
    ) -> Result<Self, QuantityValidationError> {
        let qtd = margin.as_f64() * leverage.as_f64() * price.as_f64() / SATS_PER_BTC;

        Self::try_from(qtd.floor() as u128)
    }

    /// Calculates quantity from a percentage of a given balance.
    ///
    /// Converts a balance in satoshis to USD using the provided market price, then calculates the
    /// quantity corresponding to the specified percentage of that balance.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::{OrderQuantity, Price, PercentageCapped};
    ///
    /// let balance = 10_000_000; // In sats
    /// let market_price = Price::try_from(100_000.0).unwrap(); // Price in USD/BTC
    /// let balance_perc = PercentageCapped::try_from(10.0).unwrap(); // 10%
    ///
    /// let quantity = OrderQuantity::try_from_balance_perc(
    ///     balance,
    ///     market_price,
    ///     balance_perc
    /// ).unwrap();
    ///
    /// assert_eq!(quantity.as_u64(), 1_000); // 1_000 [USD]
    /// ```
    // TODO: Remove this helper in a future release. The implementation mixes `u64` balance
    // input with `f64` price/percentage arithmetic and cannot report invalid balance input
    // precisely through `QuantityValidationError`.
    #[deprecated(
        note = "this helper is numerically ambiguous and will be removed in a future release"
    )]
    pub fn try_from_balance_perc(
        balance: u64,
        market_price: Price,
        balance_perc: PercentageCapped,
    ) -> Result<Self, QuantityValidationError> {
        let balance_usd = balance as f64 * market_price.as_f64() / SATS_PER_BTC;
        let quantity_target = balance_usd * balance_perc.as_f64() / 100.0;

        Self::try_from(quantity_target.floor())
    }
}

impl crate::sealed::Sealed for OrderQuantity {}

impl Quantity for OrderQuantity {
    fn as_f64(&self) -> f64 {
        self.as_f64()
    }
}

impl From<OrderQuantity> for u32 {
    fn from(value: OrderQuantity) -> Self {
        value.0
    }
}

impl From<OrderQuantity> for u64 {
    fn from(value: OrderQuantity) -> Self {
        value.0 as u64
    }
}

impl From<OrderQuantity> for u128 {
    fn from(value: OrderQuantity) -> Self {
        value.0 as u128
    }
}

impl From<OrderQuantity> for i64 {
    fn from(value: OrderQuantity) -> Self {
        value.0 as i64
    }
}

impl From<OrderQuantity> for i128 {
    fn from(value: OrderQuantity) -> Self {
        value.0 as i128
    }
}

impl From<OrderQuantity> for f64 {
    fn from(value: OrderQuantity) -> Self {
        value.0 as f64
    }
}

impl TryFrom<u8> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u16> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u32> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u64> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u128> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value < Self::MIN.0 as u128 {
            return Err(QuantityValidationError::TooLow {
                value: value as i128,
            });
        }

        if value > Self::MAX.0 as u128 {
            return Err(QuantityValidationError::TooHigh { value });
        }

        Ok(OrderQuantity(value as u32))
    }
}

impl TryFrom<usize> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<i8> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i16> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i32> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(quantity: i32) -> Result<Self, Self::Error> {
        Self::try_from(quantity as i128)
    }
}

impl TryFrom<i64> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i128> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: i128) -> Result<Self, Self::Error> {
        if value < Self::MIN.0 as i128 {
            return Err(QuantityValidationError::TooLow { value });
        }

        if value > Self::MAX.0 as i128 {
            return Err(QuantityValidationError::TooHigh {
                value: value as u128,
            });
        }

        Ok(OrderQuantity(value as u32))
    }
}

impl TryFrom<isize> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<f32> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::try_from(value as f64)
    }
}

impl TryFrom<f64> for OrderQuantity {
    type Error = QuantityValidationError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.fract() != 0.0 {
            return Err(QuantityValidationError::NotAnInteger { value });
        }

        Self::try_from(value as i128)
    }
}

impl fmt::Display for OrderQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for OrderQuantity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for OrderQuantity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let quantity = u32::deserialize(deserializer)?;
        OrderQuantity::try_from(quantity).map_err(|e| de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_add_quantity() {
        let base = OrderQuantity::try_from(1_000).unwrap();
        let added = OrderQuantity::try_from(500).unwrap();

        let total = base.try_add(added).unwrap();

        assert_eq!(total, OrderQuantity::try_from(1_500).unwrap());
    }

    #[test]
    fn test_try_add_quantity_fails_above_max() {
        let error = OrderQuantity::MAX
            .try_add(OrderQuantity::MIN)
            .err()
            .unwrap();

        assert!(matches!(
            error,
            QuantityValidationError::TooHigh { value }
                if value == (OrderQuantity::MAX.as_u64() + OrderQuantity::MIN.as_u64()) as u128
        ));
    }

    #[test]
    fn test_try_sub_quantity() {
        let base = OrderQuantity::try_from(1_000).unwrap();
        let removed = OrderQuantity::try_from(500).unwrap();

        let remaining = base.try_sub(removed).unwrap();

        assert_eq!(remaining, OrderQuantity::try_from(500).unwrap());
    }

    #[test]
    fn test_try_sub_quantity_fails_below_min() {
        let error = OrderQuantity::MIN
            .try_sub(OrderQuantity::MIN)
            .err()
            .unwrap();

        assert!(matches!(
            error,
            QuantityValidationError::TooLow { value } if value == 0
        ));

        let error = OrderQuantity::MIN
            .try_sub(OrderQuantity::try_from(2).unwrap())
            .err()
            .unwrap();

        assert!(matches!(
            error,
            QuantityValidationError::TooLow { value } if value == -1
        ));
    }

    #[test]
    fn test_try_add_quantity_with_primitive_integer() {
        let base = OrderQuantity::try_from(1_000).unwrap();

        let total = base.try_add(500i32).unwrap();
        assert_eq!(total, OrderQuantity::try_from(1_500).unwrap());

        let total = base.try_add(500usize).unwrap();
        assert_eq!(total, OrderQuantity::try_from(1_500).unwrap());

        let total = base.try_add(500u128).unwrap();
        assert_eq!(total, OrderQuantity::try_from(1_500).unwrap());
    }

    #[test]
    fn test_try_sub_quantity_with_primitive_integer() {
        let base = OrderQuantity::try_from(1_000).unwrap();

        let remaining = base.try_sub(500i32).unwrap();
        assert_eq!(remaining, OrderQuantity::try_from(500).unwrap());

        let remaining = base.try_sub(500usize).unwrap();
        assert_eq!(remaining, OrderQuantity::try_from(500).unwrap());

        let remaining = base.try_sub(500u128).unwrap();
        assert_eq!(remaining, OrderQuantity::try_from(500).unwrap());
    }

    #[test]
    fn test_try_add_quantity_negative_operand_reduces_value() {
        let base = OrderQuantity::try_from(1_000).unwrap();

        let total = base.try_add(-500i64).unwrap();
        assert_eq!(total, OrderQuantity::try_from(500).unwrap());
    }

    #[test]
    fn test_try_add_quantity_negative_operand_fails_when_below_min() {
        let error = OrderQuantity::try_from(500)
            .unwrap()
            .try_add(-501i64)
            .err()
            .unwrap();

        assert!(matches!(
            error,
            QuantityValidationError::TooLow { value } if value == -1
        ));
    }

    #[test]
    fn test_try_sub_quantity_negative_operand_increases_value() {
        let base = OrderQuantity::try_from(1_000).unwrap();

        let total = base.try_sub(-500i64).unwrap();
        assert_eq!(total, OrderQuantity::try_from(1_500).unwrap());
    }

    #[test]
    fn test_try_sub_quantity_large_negative_operand_reports_too_high() {
        let error = OrderQuantity::MAX.try_sub(i128::MIN).err().unwrap();

        assert!(matches!(
            error,
            QuantityValidationError::TooHigh { value } if value == u128::MAX
        ));
    }

    #[test]
    fn test_try_add_quantity_overflow_reports_true_value() {
        let error = OrderQuantity::MAX.try_add(10u32).err().unwrap();

        assert!(matches!(
            error,
            QuantityValidationError::TooHigh { value }
                if value == OrderQuantity::MAX.as_u64() as u128 + 10
        ));
    }

    #[test]
    fn test_calculate_quantity() {
        let margin = Margin::try_from(1_000).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = Leverage::try_from(1.0).unwrap();

        let quantity = OrderQuantity::try_calculate(margin, price, leverage).unwrap();
        assert_eq!(quantity, OrderQuantity::MIN);

        let margin = Margin::try_from(700).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = Leverage::try_from(2.0).unwrap();

        let quantity = OrderQuantity::try_calculate(margin, price, leverage).unwrap();
        assert_eq!(quantity, OrderQuantity::MIN);

        let margin = Margin::try_from(10).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = Leverage::try_from(100.0).unwrap();

        let quantity = OrderQuantity::try_calculate(margin, price, leverage).unwrap();
        assert_eq!(quantity, OrderQuantity::MIN);

        let margin = Margin::try_from(5_000_000).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = Leverage::try_from(100.0).unwrap();

        let quantity = OrderQuantity::try_calculate(margin, price, leverage).unwrap();
        assert_eq!(quantity, OrderQuantity::MAX);

        let margin = Margin::try_from(9).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = Leverage::try_from(100.0).unwrap();

        let quantity_validation_error = OrderQuantity::try_calculate(margin, price, leverage)
            .err()
            .unwrap();
        assert!(matches!(
            quantity_validation_error,
            QuantityValidationError::TooLow { value: _ }
        ));

        let margin = Margin::try_from(5_001_000).unwrap();
        let price = Price::try_from(100_000).unwrap();
        let leverage = Leverage::try_from(100.0).unwrap();

        let quantity_validation_error = OrderQuantity::try_calculate(margin, price, leverage)
            .err()
            .unwrap();
        assert!(matches!(
            quantity_validation_error,
            QuantityValidationError::TooHigh { value: _ }
        ));
    }
}
