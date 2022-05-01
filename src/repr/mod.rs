pub trait Repr<T>
where
    Self: Sized,
{
    /// Convert a raw type to the common representation.
    fn from(v: T) -> Self;
    /// Try to convert the representation back into one of the raw message types.
    fn try_into(self) -> Result<T, Self>;
    /// Check whether the representation can be turned into this raw type, without consuming.
    fn can_into(&self) -> bool;
}

mod json_string_repr;
pub use json_string_repr::*;
mod box_any_repr;
pub use box_any_repr::*;
