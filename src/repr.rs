use std::any::Any;

pub trait Repr<T>: Send + Sync + 'static
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


#[repr(transparent)]
pub struct DynMessage(Box<dyn Any + Send + Sync + 'static>);

/// We can turn anything into a `DynMessage`.
impl<T: 'static + Send + Sync + Unpin> Repr<T> for DynMessage {
    fn from(v: T) -> Self {
        DynMessage(Box::new(v))
    }
    fn try_into(self) -> Result<T, Self> {
        match self.0.downcast::<T>() {
            Ok(b) => Ok(*b),
            Err(e) => Err(DynMessage(e)),
        }
    }
    fn can_into(&self) -> bool {
        self.0.is::<T>()
    }
}

use serde::{de::DeserializeOwned, Serialize};

#[repr(transparent)]
pub struct MyString(pub String);

impl<T: Serialize + DeserializeOwned + 'static> Repr<T> for MyString {
    fn from(v: T) -> Self {
        MyString(serde_json::to_string(&v).unwrap())
    }

    fn try_into(self) -> Result<T, Self> {
        serde_json::from_str(&self.0).map_err(|_| self)
    }

    fn can_into(&self) -> bool {
        todo!()
    }
}
