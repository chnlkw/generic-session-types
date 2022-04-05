#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use std::{future::Future, marker::PhantomData};

pub struct Eps;
pub struct Recv<T, P>(PhantomData<(T, P)>);
pub struct Send<T, P>(PhantomData<(T, P)>);

pub trait HasDual {
    type Dual;
}

impl HasDual for Eps {
    type Dual = Eps;
}

impl<T, P: HasDual> HasDual for Recv<T, P> {
    type Dual = Send<T, P::Dual>;
}

impl<T, P: HasDual> HasDual for Send<T, P> {
    type Dual = Recv<T, P::Dual>;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Error {
    SendErr,
    RecvErr,
    ConvertErr,
}

pub trait Channel: Sized {
    type R;
    type SendFuture: Future<Output = Result<Self, Error>> + 'static
    where
        Self: 'static;
    fn send(self, t: Self::R) -> Self::SendFuture;

    type RecvFuture<'a>: Future<Output = Result<Self::R, Error>>
    where
        Self: 'a;
    fn recv(&mut self) -> Self::RecvFuture<'_>;
}

pub struct Chan<P, C: Channel>(C, PhantomData<P>);

impl<P, C: Channel> Chan<P, C> {
    fn new(c: C) -> Self {
        Self(c, PhantomData)
    }
}

impl<P, T, C: Channel> Chan<Send<T, P>, C>
where
    C: 'static,
    C::R: Repr<T> + 'static,
{
    pub fn send(self, v: T) -> impl Future<Output = Result<Chan<P, C>, Error>> + 'static {
        let m = <C::R as Repr<T>>::from(v);
        let fut = self.0.send(m);
        let r = async move {
            let c = fut.await?;
            let chan = Chan(c, PhantomData);
            Ok(chan)
        };
        i_want_static_future(r)
    }
}

impl<P, T, C: Channel> Chan<Recv<T, P>, C>
where
    C::R: Repr<T>,
{
    pub async fn recv(self) -> Result<(T, Chan<P, C>), Error> {
        let mut c = self.0;
        let v: T = Repr::try_into(c.recv().await.map_err(|_| Error::RecvErr)?)
            .map_err(|_| Error::ConvertErr)?;
        let chan = Chan(c, PhantomData);
        Ok((v, chan))
    }
}

impl<C: Channel> Chan<Eps, C> {
    pub async fn close(self) -> Result<(), Error> {
        //TODO: call c.close()
        Ok(())
    }
}

pub fn i_want_static_future<F>(a: F) -> F
where
    F: Future + 'static,
{
    a
}

mod repr;
pub use repr::{DynMessage, Repr};
pub mod mpsc;

#[cfg(test)]
mod test;
