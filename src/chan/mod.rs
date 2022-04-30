use crate::*;
use std::{future::Future, marker::PhantomData};

pub trait RawChan {
    type R;
    type SendFuture<'a>: Future<Output = Result<(), Error>> + 'a
    where
        Self: 'a;
    fn send(&mut self, r: Self::R) -> Self::SendFuture<'_>;

    type RecvFuture<'a>: Future<Output = Result<Self::R, Error>>
    where
        Self: 'a;
    fn recv(&mut self) -> Self::RecvFuture<'_>;
}

#[repr(transparent)]
#[must_use]
pub struct Chan<P: HasDual, C: RawChan>(C, PhantomData<P>);

impl<P: HasDual, C: RawChan> Chan<P, C> {
    pub fn new(c: C) -> Self {
        Self(c, PhantomData)
    }
}

impl<P: HasDual, T, C: RawChan> Chan<Send<T, P>, C>
where
    C::R: Repr<T>,
{
    pub async fn send(self, t: T) -> Result<Chan<P, C>, Error> {
        let mut c = self.0;
        let r = <C::R as Repr<T>>::from(t);
        c.send(r).await?;
        let chan = Chan(c, PhantomData);
        Ok(chan)
    }
}

impl<P: HasDual, T, C: RawChan> Chan<Recv<T, P>, C>
where
    C::R: Repr<T>,
{
    pub async fn recv(self) -> Result<(T, Chan<P, C>), Error> {
        let mut c = self.0;
        let r = c.recv().await.map_err(|_| Error::RecvErr)?;
        let t: T = repr::Repr::try_into(r).map_err(|_| Error::ConvertErr)?;
        let chan = Chan(c, PhantomData);
        Ok((t, chan))
    }
}

impl<C: RawChan> Chan<Eps, C> {
    pub async fn close(self) -> Result<(), Error> {
        //TODO: call c.close()
        Ok(())
    }
}

pub mod mpsc;
