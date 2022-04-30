use super::*;
use serde::{de::DeserializeOwned, Serialize};

#[repr(transparent)]
pub struct JsonStringRepr(pub String);

impl<T: Serialize + DeserializeOwned + 'static> Repr<T> for JsonStringRepr {
    fn from(v: T) -> Self {
        JsonStringRepr(serde_json::to_string(&v).unwrap())
    }

    fn try_into(self) -> Result<T, Self> {
        serde_json::from_str(&self.0).map_err(|_| self)
    }

    fn can_into(&self) -> bool {
        todo!()
    }
}
