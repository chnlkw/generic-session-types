#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use std::future::Future;

use generic_session_types::*;

// #[derive(Choose)]
enum Proto1 {
    P1(Send<u32, Close>),    // 1
    P2(Recv<String, Close>), // 2
    P3(Close),               // 3
}

/* begin proc macro gen */
struct Proto1Dual {
    p1: <Send<u32, Close> as HasDual>::Dual,
    p2: <Recv<String, Close> as HasDual>::Dual,
    p3: <Close as HasDual>::Dual,
}

impl HasDual for Proto1 {
    type Dual = Proto1Dual;
}

impl Choose3 for Proto1 {
    type T1 = Send<u32, Close>;
    type T2 = Recv<String, Close>;
    type T3 = Close;
}

impl HasDual for Proto1Dual {
    type Dual = Proto1;
}
trait Proto1ChanExt {
    type C: RawChan;
    type P1Future: Future<Output = Result<Chan<Send<u32, Close>, Self::C>, Error>> + 'static
    where
        Self: 'static;
    fn p1(self) -> Self::P1Future;
}

impl<C: RawChan> Proto1ChanExt for Chan<Proto1, C>
where
    C::R: Repr<u8>,
{
    type C = C;

    type P1Future = impl Future<Output = Result<Chan<Send<u32, Close>, Self::C>, Error>> + 'static where Self:'static;
    fn p1(self) -> Self::P1Future {
        let mut c = self.into_raw();
        async move {
            c.send(<C::R as Repr<u8>>::from(1)).await?;
            Ok(Chan::from_raw(c))
        }
    }
}

pub enum Proto1DualOffer<C: RawChan> {
    P1(Chan<<Send<u32, Close> as HasDual>::Dual, C>),
    P2(Chan<<Recv<String, Close> as HasDual>::Dual, C>),
    P3(Chan<<Close as HasDual>::Dual, C>),
}

trait Proto1DualChanExt {
    type C: RawChan;
    type OfferFuture: Future<Output = Result<Proto1DualOffer<Self::C>, Error>> + 'static
    where
        Self: 'static;
    fn offer(self) -> Self::OfferFuture;
}

impl<C: RawChan> Proto1DualChanExt for Chan<Proto1Dual, C>
where
    C::R: Repr<u8>,
{
    type C = C;

    type OfferFuture = impl Future<Output = Result<Proto1DualOffer<Self::C>, Error>> + 'static where Self:'static;
    fn offer(self) -> Self::OfferFuture {
        let mut c = self.into_raw();
        async move {
            let r = c.recv().await.map_err(|_| Error::RecvErr)?;
            let t: u8 = Repr::try_into(r).map_err(|_| Error::ConvertErr)?;
            match t {
                1 => Ok(Proto1DualOffer::P1(Chan::from_raw(c))),
                _ => Err(Error::ConvertErr),
            }
        }
    }
}
/* end proc macro gen */

async fn server(c: Chan<Proto1, impl RawChan<R = BoxAnyRepr> + 'static>) -> Result<(), Error> {
    let c = c.p1().await?;
    let c = c.send(1).await?;
    c.close().await?;
    Ok(())
}

#[tokio::test]
async fn t1() {}
