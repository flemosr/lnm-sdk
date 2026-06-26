use std::{convert::TryFrom, fmt, num::NonZeroU64};

use serde::{Deserialize, Serialize, de};

use super::{
    SATS_PER_BTC,
    error::{MarginValidationError, TradeValidationError},
    leverage::Leverage,
    price::Price,
    quantity::{Quantity, order::OrderQuantity},
    trade::TradeSide,
};

/// A validated margin value denominated in satoshis.
///
/// Margin represents the collateral required to open a leveraged trading position.
/// This type ensures that margin values meet minimum requirements and can be safely used when
/// trading futures.
///
/// Margin values must be integers greater than or equal to [`Margin::MIN`] (1 satoshi).
///
/// # Examples
///
/// ```
/// use lnm_sdk::rest::v3::models::Margin;
///
/// // Create a margin value from satoshis
/// let margin = Margin::try_from(10_000).unwrap();
/// assert_eq!(margin.as_u64(), 10_000);
///
/// // Values below the minimum will fail
/// assert!(Margin::try_from(0).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Margin(u64);

impl Margin {
    /// The minimum allowed margin value (1 satoshi).
    pub const MIN: Self = Self(1);

    /// The maximum allowed margin value (21,000,000 BTC, or 2.1 quadrillion satoshis).
    ///
    /// This limit keeps every valid `Margin` value exactly representable as an `f64`, since
    /// `2_100_000_000_000_000 < 2^53`.
    pub const MAX: Self = Self(2_100_000_000_000_000);

    /// Creates a `Margin` by rounding and bounding the given value to the valid range.
    ///
    /// This method rounds the input to the nearest integer and ensures it is at least
    /// [`Margin::MIN`].
    /// It should be used to ensure a valid `Margin` without error handling.
    ///
    /// **Note:** In order to check whether a value is a valid margin and receive an error for
    /// invalid values, use [`Margin::try_from`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::Margin;
    ///
    /// // Values above minimum are rounded
    /// let m = Margin::bounded(5_000.7);
    /// assert_eq!(m.as_u64(), 5_001);
    ///
    /// // Values below minimum are bounded to MIN
    /// let m = Margin::bounded(-1);
    /// assert_eq!(m, Margin::MIN);
    ///
    /// // Zero becomes MIN
    /// let m = Margin::bounded(0);
    /// assert_eq!(m, Margin::MIN);
    /// ```
    pub fn bounded<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        let as_f64: f64 = value.into();
        let rounded = as_f64.round().max(0.0) as u64;
        let clamped = rounded.clamp(Self::MIN.0, Self::MAX.0);

        Self(clamped)
    }

    /// Returns the margin value as its underlying `u64` representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::Margin;
    ///
    /// let margin = Margin::try_from(10_000).unwrap();
    /// assert_eq!(margin.as_u64(), 10_000);
    /// ```
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Returns the margin value as an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::Margin;
    ///
    /// let margin = Margin::try_from(10_000).unwrap();
    /// assert_eq!(margin.as_i64(), 10_000);
    /// ```
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }

    /// Returns the margin value as a `f64`.
    ///
    /// Because `Margin::MAX` is below `2^53`, every valid margin value is represented exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::Margin;
    ///
    /// let margin = Margin::try_from(10_000).unwrap();
    /// assert_eq!(margin.as_f64(), 10_000.0);
    /// ```
    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    /// Adds a signed integer amount to this margin and validates the result.
    ///
    /// The operand may be any primitive integer type, including negative values. Only the final
    /// result is validated against [`Margin::MIN`] and [`Margin::MAX`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::Margin;
    ///
    /// let base = Margin::try_from(10_000).unwrap();
    ///
    /// // Add another `Margin`
    /// let added = Margin::try_from(5_000).unwrap();
    /// let total = base.try_add(added).unwrap();
    /// assert_eq!(total.as_u64(), 15_000);
    ///
    /// // Add a raw integer
    /// let total = base.try_add(5_000i64).unwrap();
    /// assert_eq!(total.as_u64(), 15_000);
    ///
    /// // Subtract via a negative operand
    /// let total = base.try_add(-5_000i64).unwrap();
    /// assert_eq!(total.as_u64(), 5_000);
    /// ```
    pub fn try_add(self, other: impl TryInto<i128>) -> Result<Self, MarginValidationError> {
        let other = other
            .try_into()
            .map_err(|_| MarginValidationError::TooHigh { value: u128::MAX })?;
        let sum = i128::from(self)
            .checked_add(other)
            .ok_or(MarginValidationError::TooHigh { value: u128::MAX })?;

        Self::try_from(sum)
    }

    /// Subtracts a signed integer amount from this margin and validates the result.
    ///
    /// The operand may be any primitive integer type, including negative values. Only the final
    /// result is validated against [`Margin::MIN`] and [`Margin::MAX`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::Margin;
    ///
    /// let base = Margin::try_from(10_000).unwrap();
    ///
    /// // Subtract another `Margin`
    /// let removed = Margin::try_from(5_000).unwrap();
    /// let remaining = base.try_sub(removed).unwrap();
    /// assert_eq!(remaining.as_u64(), 5_000);
    ///
    /// // Subtract a raw integer
    /// let remaining = base.try_sub(5_000i64).unwrap();
    /// assert_eq!(remaining.as_u64(), 5_000);
    ///
    /// // Add via a negative operand
    /// let remaining = base.try_sub(-5_000i64).unwrap();
    /// assert_eq!(remaining.as_u64(), 15_000);
    /// ```
    pub fn try_sub(self, other: impl TryInto<i128>) -> Result<Self, MarginValidationError> {
        let other = other
            .try_into()
            .map_err(|_| MarginValidationError::TooLow { value: i128::MIN })?;
        let difference = i128::from(self)
            .checked_sub(other)
            .ok_or(MarginValidationError::TooHigh { value: u128::MAX })?;

        Self::try_from(difference)
    }

    /// Calculates margin from quantity (USD), price (BTC/USD), and leverage.
    ///
    /// The margin is calculated using the formula:
    ///
    /// margin = (quantity * SATS_PER_BTC) / (price * leverage)
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::{Margin, OrderQuantity, Price, Leverage};
    ///
    /// let quantity = OrderQuantity::try_from(1_000).unwrap();
    /// let price = Price::try_from(100_000.0).unwrap();
    /// let leverage = Leverage::try_from(10.0).unwrap();
    ///
    /// let margin = Margin::calculate(quantity, price, leverage);
    /// ```
    pub fn calculate(quantity: OrderQuantity, price: Price, leverage: Leverage) -> Self {
        let margin = quantity.as_f64() * (SATS_PER_BTC / (price.as_f64() * leverage.as_f64()));

        Self::try_from(margin.ceil() as u64).expect("must result in valid `Margin`")
    }

    /// Estimates margin from a target liquidation price.
    ///
    /// Calculates the required margin to achieve a specific liquidation price for a position
    /// with the given quantity and entry price.
    ///
    /// + For long positions (Buy): liquidation price must be below entry price
    /// + For short positions (Sell): liquidation price must be above entry price
    ///
    /// # Examples
    ///
    /// ```
    /// use lnm_sdk::rest::v3::models::{Margin, OrderQuantity, Price, TradeSide};
    ///
    /// let quantity = OrderQuantity::try_from(1_000).unwrap();
    /// let entry_price = Price::try_from(100_000.0).unwrap();
    /// let liquidation_price = Price::try_from(95_000.0).unwrap();
    ///
    /// let margin = Margin::est_from_liquidation_price(
    ///     TradeSide::Buy,
    ///     quantity,
    ///     entry_price,
    ///     liquidation_price
    /// ).unwrap();
    /// ```
    pub fn est_from_liquidation_price(
        side: TradeSide,
        quantity: impl Quantity,
        price: Price,
        liquidation: Price,
    ) -> Result<Self, TradeValidationError> {
        match side {
            TradeSide::Buy if liquidation >= price => {
                return Err(TradeValidationError::LiquidationNotBelowPriceForLong {
                    liquidation,
                    price,
                });
            }
            TradeSide::Sell if liquidation <= price => {
                return Err(TradeValidationError::LiquidationNotAbovePriceForShort {
                    liquidation,
                    price,
                });
            }
            _ => {}
        }

        // Calculate 'a' and 'b' from `trade_utils::est_liquidation_from_leverage`

        let a = 1.0 / price.as_f64();

        let b = match side {
            TradeSide::Buy => {
                // liquidation_price = 1.0 / (a + b)
                1.0 / liquidation.as_f64() - a
            }
            TradeSide::Sell => {
                // liquidation_price = 1.0 / (a - b)
                a - 1.0 / liquidation.as_f64()
            }
        };

        assert!(b > 0.0, "'b' must be positive from validations above");

        let floored_margin = b * SATS_PER_BTC * quantity.as_f64();

        let margin =
            Margin::try_from(floored_margin.ceil() as u64).expect("must be valid `Margin`");

        Ok(margin)
    }
}

impl From<Margin> for u64 {
    fn from(value: Margin) -> Self {
        value.0
    }
}

impl From<Margin> for u128 {
    fn from(value: Margin) -> Self {
        value.0 as u128
    }
}

impl From<Margin> for i64 {
    fn from(value: Margin) -> Self {
        value.0 as i64
    }
}

impl From<Margin> for i128 {
    fn from(value: Margin) -> Self {
        value.0 as i128
    }
}

impl From<Margin> for f64 {
    fn from(value: Margin) -> Self {
        value.0 as f64
    }
}

impl TryFrom<u8> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u16> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u32> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u64> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<u128> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if value < Self::MIN.0 as u128 {
            return Err(MarginValidationError::TooLow {
                value: value as i128,
            });
        }

        if value > Self::MAX.0 as u128 {
            return Err(MarginValidationError::TooHigh { value });
        }

        Ok(Margin(value as u64))
    }
}

impl TryFrom<usize> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::try_from(value as u128)
    }
}

impl TryFrom<NonZeroU64> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: NonZeroU64) -> Result<Self, Self::Error> {
        Self::try_from(value.get())
    }
}

impl TryFrom<i8> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i16> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i32> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i64> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<i128> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: i128) -> Result<Self, Self::Error> {
        if value < Self::MIN.0 as i128 {
            return Err(MarginValidationError::TooLow { value });
        }

        if value > Self::MAX.0 as i128 {
            return Err(MarginValidationError::TooHigh {
                value: value as u128,
            });
        }

        Ok(Margin(value as u64))
    }
}

impl TryFrom<isize> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Self::try_from(value as i128)
    }
}

impl TryFrom<f32> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::try_from(value as f64)
    }
}

impl TryFrom<f64> for Margin {
    type Error = MarginValidationError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(MarginValidationError::NotFinite);
        }

        if value.fract() != 0.0 {
            return Err(MarginValidationError::NotAnInteger { value });
        }

        Self::try_from(value as i128)
    }
}

impl fmt::Display for Margin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Margin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for Margin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let margin_u64 = u64::deserialize(deserializer)?;
        Margin::try_from(margin_u64).map_err(|e| de::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::trade::util as trade_util;

    use super::*;

    #[test]
    fn test_try_add_margin() {
        let base = Margin::try_from(10_000).unwrap();
        let added = Margin::try_from(5_000).unwrap();

        let total = base.try_add(added).unwrap();

        assert_eq!(total, Margin::try_from(15_000).unwrap());
    }

    #[test]
    fn test_try_add_margin_fails_above_max() {
        let error = Margin::MAX.try_add(Margin::MIN).err().unwrap();

        assert!(matches!(
            error,
            MarginValidationError::TooHigh { value }
                if value == Margin::MAX.as_u64() as u128 + 1
        ));
    }

    #[test]
    fn test_try_sub_margin() {
        let base = Margin::try_from(10_000).unwrap();
        let removed = Margin::try_from(5_000).unwrap();

        let remaining = base.try_sub(removed).unwrap();

        assert_eq!(remaining, Margin::try_from(5_000).unwrap());
    }

    #[test]
    fn test_try_sub_margin_fails_below_min() {
        let error = Margin::MIN.try_sub(Margin::MIN).err().unwrap();

        assert!(matches!(
            error,
            MarginValidationError::TooLow { value } if value == 0
        ));

        let error = Margin::MIN
            .try_sub(Margin::try_from(2).unwrap())
            .err()
            .unwrap();

        assert!(matches!(
            error,
            MarginValidationError::TooLow { value } if value == -1
        ));
    }

    #[test]
    fn test_try_add_margin_with_primitive_integer() {
        let base = Margin::try_from(10_000).unwrap();

        let total = base.try_add(5_000u32).unwrap();
        assert_eq!(total, Margin::try_from(15_000).unwrap());

        let total = base.try_add(5_000usize).unwrap();
        assert_eq!(total, Margin::try_from(15_000).unwrap());

        let total = base.try_add(5_000u128).unwrap();
        assert_eq!(total, Margin::try_from(15_000).unwrap());
    }

    #[test]
    fn test_try_sub_margin_with_primitive_integer() {
        let base = Margin::try_from(10_000).unwrap();

        let remaining = base.try_sub(5_000u32).unwrap();
        assert_eq!(remaining, Margin::try_from(5_000).unwrap());

        let remaining = base.try_sub(5_000usize).unwrap();
        assert_eq!(remaining, Margin::try_from(5_000).unwrap());

        let remaining = base.try_sub(5_000u128).unwrap();
        assert_eq!(remaining, Margin::try_from(5_000).unwrap());
    }

    #[test]
    fn test_try_add_margin_negative_operand_reduces_value() {
        let base = Margin::try_from(10_000).unwrap();

        let total = base.try_add(-5_000i64).unwrap();
        assert_eq!(total, Margin::try_from(5_000).unwrap());
    }

    #[test]
    fn test_try_add_margin_negative_operand_fails_when_below_min() {
        let error = Margin::try_from(500)
            .unwrap()
            .try_add(-501i64)
            .err()
            .unwrap();

        assert!(matches!(
            error,
            MarginValidationError::TooLow { value } if value == -1
        ));
    }

    #[test]
    fn test_try_sub_margin_negative_operand_increases_value() {
        let base = Margin::try_from(10_000).unwrap();

        let total = base.try_sub(-5_000i64).unwrap();
        assert_eq!(total, Margin::try_from(15_000).unwrap());
    }

    #[test]
    fn test_try_sub_margin_large_negative_operand_reports_too_high() {
        let error = Margin::MAX.try_sub(i128::MIN).err().unwrap();

        assert!(matches!(
            error,
            MarginValidationError::TooHigh { value } if value == u128::MAX
        ));
    }

    #[test]
    fn test_margin_try_from_u128() {
        let error = Margin::try_from(0u128).err().unwrap();
        assert!(matches!(error, MarginValidationError::TooLow { value } if value == 0));

        let error = Margin::try_from(u128::MAX).err().unwrap();
        assert!(matches!(error, MarginValidationError::TooHigh { value } if value == u128::MAX));

        let error = Margin::try_from(Margin::MAX.as_u64() as u128 + 1)
            .err()
            .unwrap();
        assert!(matches!(
            error,
            MarginValidationError::TooHigh { value }
                if value == Margin::MAX.as_u64() as u128 + 1
        ));

        assert_eq!(
            Margin::try_from(Margin::MAX.as_u64() as u128).unwrap(),
            Margin::MAX
        );
    }

    #[test]
    fn test_margin_max_is_exactly_representable_as_f64() {
        let max_f64 = Margin::MAX.as_f64();
        let back = Margin::try_from(max_f64).unwrap();
        assert_eq!(back, Margin::MAX);
    }

    #[test]
    fn test_margin_try_from_non_zero_u64() {
        let margin = Margin::try_from(NonZeroU64::new(10_000).unwrap()).unwrap();
        assert_eq!(margin, Margin::try_from(10_000).unwrap());

        let error = Margin::try_from(NonZeroU64::new(Margin::MAX.as_u64() + 1).unwrap())
            .err()
            .unwrap();
        assert!(
            matches!(error, MarginValidationError::TooHigh { value } if value == Margin::MAX.as_u64() as u128 + 1)
        );
    }

    #[test]
    fn test_calculate_margin() {
        let quantity = OrderQuantity::try_from(5).unwrap();
        let price = Price::try_from(95000).unwrap();
        let leverage = Leverage::try_from(1.0).unwrap();

        let margin = Margin::calculate(quantity, price, leverage);
        assert_eq!(margin.as_u64(), 5264);

        let leverage = Leverage::try_from(2.0).unwrap();
        let margin = Margin::calculate(quantity, price, leverage);
        assert_eq!(margin.as_u64(), 2632);

        let leverage = Leverage::try_from(50.0).unwrap();
        let margin = Margin::calculate(quantity, price, leverage);
        assert_eq!(margin.as_u64(), 106);

        let leverage = Leverage::try_from(100.0).unwrap();
        let margin = Margin::calculate(quantity, price, leverage);
        assert_eq!(margin.as_u64(), 53);

        // Edge case: Min margin
        let margin = Margin::calculate(OrderQuantity::MIN, Price::MAX, Leverage::MAX);
        assert_eq!(margin, Margin::MIN);

        // Edge case: Max reachable margin
        let margin = Margin::calculate(OrderQuantity::MAX, Price::MIN, Leverage::MIN);
        assert_eq!(margin.as_u64(), 50_000_000_000_000);
    }

    #[test]
    fn test_margin_from_liquidation_price_calculation() {
        // Test case 1: Buy side with low leverage

        let side = TradeSide::Buy;
        let quantity = OrderQuantity::try_from(1_000).unwrap();
        let entry_price = Price::try_from(100_000).unwrap();
        let leverage = Leverage::MIN;

        let liquidation_price =
            trade_util::est_liquidation_from_leverage(side, quantity, entry_price, leverage);
        let margin =
            Margin::est_from_liquidation_price(side, quantity, entry_price, liquidation_price)
                .expect("should calculate valid margin");
        let expected_margin = Margin::calculate(quantity, entry_price, leverage);

        assert!(
            (margin.as_i64() - expected_margin.as_i64()).abs() <= 999,
            "Margin difference too large: calculated {} vs expected {}",
            margin.as_u64(),
            expected_margin.as_u64()
        );

        // Test case 2: Buy side with high leverage

        let leverage = Leverage::MAX;
        let liquidation_price =
            trade_util::est_liquidation_from_leverage(side, quantity, entry_price, leverage);
        let margin =
            Margin::est_from_liquidation_price(side, quantity, entry_price, liquidation_price)
                .expect("should calculate valid margin");
        let expected_margin = Margin::calculate(quantity, entry_price, leverage);

        assert!(
            (margin.as_i64() - expected_margin.as_i64()).abs() <= 999,
            "Margin difference too large: calculated {} vs expected {}",
            margin.as_u64(),
            expected_margin.as_u64()
        );

        // Test case 3: Sell side with low leverage

        let side = TradeSide::Sell;
        let leverage = Leverage::MIN;
        let liquidation_price =
            trade_util::est_liquidation_from_leverage(side, quantity, entry_price, leverage);
        let margin =
            Margin::est_from_liquidation_price(side, quantity, entry_price, liquidation_price)
                .expect("should calculate valid margin");
        let expected_margin = Margin::calculate(quantity, entry_price, leverage);

        assert!(
            (margin.as_i64() - expected_margin.as_i64()).abs() <= 999,
            "Margin difference too large: calculated {} vs expected {}",
            margin.as_u64(),
            expected_margin.as_u64()
        );
    }
}
