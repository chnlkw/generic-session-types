use std::future::Future;
use tokio::sync::mpsc;

use crate::{Chan, Channel, Error, HasDual, i_want_static_future};

pub struct ChannelMpsc<T> {
    sx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
}

impl<T: 'static> ChannelMpsc<T> {
    pub fn ss(self, t: T) -> impl Future<Output = Result<Self, Error>> {
        async move {
            self.sx.send(t).await.map_err(|_| Error::SendErr)?;
            Ok(self)
        }
    }
}

impl<T: Sync + Send + 'static> Channel for ChannelMpsc<T> {
    type SendFuture = impl Future<Output = Result<Self, Error>> + 'static;
    type R = T;

    fn send(self, t: T) -> Self::SendFuture {
        let s = self;
        let fut = s.ss(t);
        i_want_static_future(fut)
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
