use std::future::Future;
use tokio::sync::mpsc;

use crate::{Chan, Error, HasDual, RawChan};

pub struct Mpsc<T> {
    sx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
}

impl<R: Sync + Send + 'static> RawChan for Mpsc<R> {
    type R = R;
    type SendFuture<'a> = impl Future<Output = Result<(), Error>> + 'a
    where
        Self: 'a;
    fn send(&mut self, r: R) -> Self::SendFuture<'_> {
        async move {
            self.sx.send(r).await.map_err(|_| Error::SendErr)?;
            Ok(())
        }
    }

    type RecvFuture<'a> = impl Future<Output = Result<R, Error>>  + 'a where Self: 'a;
    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async {
            let data = self.rx.recv().await.ok_or(Error::RecvErr)?;
            Ok(data)
        }
    }

    type CloseFuture = impl Future<Output = Result<(), Error>> + 'static;
    fn close(self) -> Self::CloseFuture {
        drop(self);
        async { Ok(()) }
    }
}

pub fn channel<P: HasDual, R: Sync + Send + 'static>(
    buffer: usize,
) -> (Chan<P, (), Mpsc<R>>, Chan<P::Dual, (), Mpsc<R>>) {
    let (sx0, rx0) = mpsc::channel(buffer);
    let (sx1, rx1) = mpsc::channel(buffer);
    let c0 = Mpsc::<R> { sx: sx0, rx: rx1 };
    let c1 = Mpsc::<R> { sx: sx1, rx: rx0 };
    (Chan::from_raw(c0), Chan::from_raw(c1))
}
