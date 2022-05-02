#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use std::future::Future;

use generic_session_types::*;

enum Proto1 {
    P1(Send<u32, Close>),
    P2(Recv<String, Close>),
    P3(Close),
}

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
        self.choose1()
    }
}

async fn server(c: Chan<Proto1, impl RawChan<R = BoxAnyRepr> + 'static>) -> Result<(), Error> {
    let c = c.p1().await?;
    let c = c.send(1).await?;
    c.close().await?;
    Ok(())
}

#[tokio::test]
async fn t1() {}
