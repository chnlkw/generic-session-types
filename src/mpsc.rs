use std::future::Future;
use tokio::sync::mpsc;

use crate::{Chan, Channel, Error, HasDual};

pub struct ChannelMpsc<T> {
    sx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
}

impl<T: Sync + Send + 'static> Channel for ChannelMpsc<T> {
    type R = T;
    type SendFuture<'a> = impl Future<Output = Result<(), Error>> + 'a
    where
        Self: 'a;
    fn send(&mut self, t: Self::R) -> Self::SendFuture<'_> {
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

pub fn session_channel<P: HasDual, R: Sync + Send + 'static>(
    buffer: usize,
) -> (
    crate::Chan<P, ChannelMpsc<R>>,
    crate::Chan<P::Dual, ChannelMpsc<R>>,
) {
    let (sx0, rx0) = mpsc::channel(buffer);
    let (sx1, rx1) = mpsc::channel(buffer);
    let c0 = ChannelMpsc::<R> { sx: sx0, rx: rx1 };
    let c1 = ChannelMpsc::<R> { sx: sx1, rx: rx0 };
    (Chan::new(c0), Chan::new(c1))
}
