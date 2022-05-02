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

    type CloseFuture: Future<Output = Result<(), Error>> + 'static;
    fn close(self) -> Self::CloseFuture;
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

impl<C: RawChan, C3: Choose3> Chan<C3, C>
where
    C::R: Repr<u8>,
{
    pub async fn choose1(self) -> Result<Chan<C3::T1, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<u8>>::from(1)).await?;
        Ok(Chan(c, PhantomData))
    }
    pub async fn choose2(self) -> Result<Chan<C3::T2, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<u8>>::from(2)).await?;
        Ok(Chan(c, PhantomData))
    }
    pub async fn choose3(self) -> Result<Chan<C3::T3, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<u8>>::from(3)).await?;
        Ok(Chan(c, PhantomData))
    }
}

impl<P: HasDual, Q: HasDual, C: RawChan> Chan<Choose<P, Q>, C>
where
    C::R: Repr<bool>,
{
    pub async fn left(self) -> Result<Chan<P, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<bool>>::from(false)).await?;
        Ok(Chan(c, PhantomData))
    }
    pub async fn right(self) -> Result<Chan<Q, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<bool>>::from(false)).await?;
        Ok(Chan(c, PhantomData))
    }
}

pub enum Branch<L, R> {
    Left(L),
    Right(R),
}

impl<P: HasDual, Q: HasDual, C: RawChan> Chan<Offer<P, Q>, C>
where
    C::R: Repr<bool>,
{
    pub async fn offer(self) -> Result<Branch<Chan<P, C>, Chan<Q, C>>, Error> {
        let mut c = self.0;
        let r = c.recv().await.map_err(|_| Error::RecvErr)?;
        let b = repr::Repr::try_into(r).map_err(|_| Error::ConvertErr)?;
        match b {
            false => Ok(Branch::Left(Chan(c, PhantomData))),
            true => Ok(Branch::Right(Chan(c, PhantomData))),
        }
    }
}

impl<C: RawChan> Chan<Close, C> {
    pub async fn close(self) -> Result<(), Error> {
        //TODO: call c.close()
        Ok(())
    }
}

pub mod mpsc;
