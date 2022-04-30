#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]


mod session;
pub use session::*;

mod chan;
pub use chan::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Error {
    SendErr,
    RecvErr,
    ConvertErr,
}

mod repr;
pub use repr::*;

#[cfg(test)]
mod test;
