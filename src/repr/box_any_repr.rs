use super::*;
use std::any::Any;

#[repr(transparent)]
pub struct BoxAnyRepr(Box<dyn Any + Send + Sync + 'static>);

impl<T: Send + Sync + 'static> Repr<T> for BoxAnyRepr {
    fn from(v: T) -> Self {
        BoxAnyRepr(Box::new(v))
    }

    fn try_into(self) -> Result<T, Self> {
        match self.0.downcast::<T>() {
            Ok(b) => Ok(*b),
            Err(e) => Err(BoxAnyRepr(e)),
        }
    }

    fn can_into(&self) -> bool {
        self.0.is::<T>()
    }
}
