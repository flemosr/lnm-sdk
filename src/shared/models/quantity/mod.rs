pub(crate) mod cross;
pub(crate) mod order;

/// A validated quantity value used by trade calculations.
pub trait Quantity: crate::sealed::Sealed + Clone + Copy + PartialEq + Eq {
    /// Returns the quantity value as a `f64`.
    fn as_f64(&self) -> f64;
}
