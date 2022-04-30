use crate::*;
use std::{future::Future, marker::PhantomData};

pub trait Channel<R> {
    type SendFuture<'a>: Future<Output = Result<(), Error>> + 'a
    where
        Self: 'a;
    fn send(&mut self, r: R) -> Self::SendFuture<'_>;

    type RecvFuture<'a>: Future<Output = Result<R, Error>>
    where
        Self: 'a;
    fn recv(&mut self) -> Self::RecvFuture<'_>;
}

pub struct Chan<P, R, C: Channel<R>>(C, PhantomData<(P, R)>);

impl<P, R, C: Channel<R>> Chan<P, R, C> {
    pub fn new(c: C) -> Self {
        Self(c, PhantomData)
    }
}

impl<P, T, R: 'static, C: Channel<R>> Chan<Send<T, P>, R, C>
where
    C: 'static,
    R: Repr<T>,
{
    pub fn send(self, v: T) -> impl Future<Output = Result<Chan<P, R, C>, Error>> + 'static {
        let mut c = self.0;
        let m = <R as Repr<T>>::from(v);
        async move {
            c.send(m).await?;
            let chan = Chan(c, PhantomData);
            Ok(chan)
        }
    }
}

impl<P, T, R: Repr<T>, C: Channel<R>> Chan<Recv<T, P>, R, C> {
    pub async fn recv(self) -> Result<(T, Chan<P, R, C>), Error> {
        let mut c = self.0;
        let v: T = Repr::try_into(c.recv().await.map_err(|_| Error::RecvErr)?)
            .map_err(|_| Error::ConvertErr)?;
        let chan = Chan(c, PhantomData);
        Ok((v, chan))
    }
}

impl<R, C: Channel<R>> Chan<Eps, R, C> {
    pub async fn close(self) -> Result<(), Error> {
        //TODO: call c.close()
        Ok(())
    }
}

pub mod mpsc;
