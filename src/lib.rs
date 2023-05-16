#![feature(impl_trait_in_assoc_type)]

mod session;
pub use session::*;

mod chan;
pub use chan::*;

#[derive(thiserror::Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Error {
    #[error("send error")]
    SendErr,
    #[error("recv error")]
    RecvErr,
    #[error("repr convert error")]
    ConvertErr,
}

mod repr;
pub use repr::*;
