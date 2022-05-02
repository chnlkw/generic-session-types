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
impl HasDual for Proto1Dual {
    type Dual = Proto1;
}

trait Proto1ChanExt : RawChan {
    fn foo(&self) {}
}

impl<C: RawChan> Proto1ChanExt for Chan<Proto1, C> {}

async fn server(c: Chan<Proto1, impl RawChan<R = BoxAnyRepr>>) {
    c.foo()
}

#[tokio::test]
async fn t1() {}
