use std::future::Future;
use tokio::sync::mpsc;

use crate::{Chan, Channel, Error, HasDual};

pub struct Mpsc<T> {
    sx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
}

impl<T: Sync + Send + 'static> Channel<T> for Mpsc<T> {
    type SendFuture<'a> = impl Future<Output = Result<(), Error>> + 'a
    where
        Self: 'a;
    fn send(&mut self, t: T) -> Self::SendFuture<'_> {
        async move {
            self.sx.send(t).await.map_err(|_| Error::SendErr)?;
            Ok(())
        }
    }

    type RecvFuture<'a> = impl Future<Output = Result<T, Error>>  + 'a where Self: 'a;
    fn recv(&mut self) -> Self::RecvFuture<'_> {
        async {
            let data = self.rx.recv().await.ok_or(Error::RecvErr)?;
            Ok(data)
        }
    }
}

pub fn channel<P: HasDual, R: Sync + Send + 'static>(
    buffer: usize,
) -> (Chan<P, R, Mpsc<R>>, Chan<P::Dual, R, Mpsc<R>>) {
    let (sx0, rx0) = mpsc::channel(buffer);
    let (sx1, rx1) = mpsc::channel(buffer);
    let c0 = Mpsc::<R> { sx: sx0, rx: rx1 };
    let c1 = Mpsc::<R> { sx: sx1, rx: rx0 };
    (Chan::new(c0), Chan::new(c1))
}
