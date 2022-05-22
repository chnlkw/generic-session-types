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
pub struct Chan<P: HasDual, E, C: RawChan>(C, PhantomData<(P, E)>);

impl<P: HasDual, E, C: RawChan> Chan<P, E, C> {
    pub fn from_raw(c: C) -> Self {
        Self(c, PhantomData)
    }
    pub fn into_raw(self) -> C {
        self.0
    }
}

impl<P: HasDual, E, T, C: RawChan> Chan<Send<T, P>, E, C>
where
    C::R: Repr<T>,
{
    pub async fn send(self, t: T) -> Result<Chan<P, E, C>, Error> {
        let mut c = self.0;
        let r = <C::R as Repr<T>>::from(t);
        c.send(r).await?;
        let chan = Chan(c, PhantomData);
        Ok(chan)
    }
}

impl<P: HasDual, E, T, C: RawChan> Chan<Recv<T, P>, E, C>
where
    C::R: Repr<T>,
{
    pub async fn recv(self) -> Result<(T, Chan<P, E, C>), Error> {
        let mut c = self.0;
        let r = c.recv().await.map_err(|_| Error::RecvErr)?;
        let t: T = repr::Repr::try_into(r).map_err(|_| Error::ConvertErr)?;
        let chan = Chan(c, PhantomData);
        Ok((t, chan))
    }
}

impl<C: RawChan, E, C3: Choose3> Chan<C3, E, C>
where
    C::R: Repr<u8>,
{
    pub async fn choose1(self) -> Result<Chan<C3::T1, E, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<u8>>::from(1)).await?;
        Ok(Chan(c, PhantomData))
    }
    pub async fn choose2(self) -> Result<Chan<C3::T2, E, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<u8>>::from(2)).await?;
        Ok(Chan(c, PhantomData))
    }
    pub async fn choose3(self) -> Result<Chan<C3::T3, E, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<u8>>::from(3)).await?;
        Ok(Chan::from_raw(c))
    }
}

impl<P: HasDual, Q: HasDual, E, C: RawChan> Chan<Choose<P, Q>, E, C>
where
    C::R: Repr<bool>,
{
    pub async fn left(self) -> Result<Chan<P, E, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<bool>>::from(false)).await?;
        Ok(Chan(c, PhantomData))
    }
    pub async fn right(self) -> Result<Chan<Q, E, C>, Error> {
        let mut c = self.0;
        c.send(<C::R as Repr<bool>>::from(false)).await?;
        Ok(Chan(c, PhantomData))
    }
}

pub enum Branch<L, R> {
    Left(L),
    Right(R),
}

impl<P: HasDual, Q: HasDual, E, C: RawChan> Chan<Offer<P, Q>, E, C>
where
    C::R: Repr<bool>,
{
    pub async fn offer(self) -> Result<Branch<Chan<P, E, C>, Chan<Q, E, C>>, Error> {
        let mut c = self.0;
        let r = c.recv().await.map_err(|_| Error::RecvErr)?;
        let b = repr::Repr::try_into(r).map_err(|_| Error::ConvertErr)?;
        match b {
            false => Ok(Branch::Left(Chan(c, PhantomData))),
            true => Ok(Branch::Right(Chan(c, PhantomData))),
        }
    }
}

impl<P: HasDual, E, C: RawChan> Chan<Rec<P>, E, C> {
    pub fn rec(self) -> Chan<P, (P, E), C> {
        Chan::from_raw(self.into_raw())
    }
}

impl<P: HasDual, E, C: RawChan> Chan<Var<Z>, (P, E), C> {
    pub fn zero(self) -> Chan<P, (P, E), C> {
        Chan::from_raw(self.into_raw())
    }
}

impl<P: HasDual, E, N, C: RawChan> Chan<Var<S<N>>, (P, E), C>
where
    Var<N>: HasDual,
{
    pub fn succ(self) -> Chan<Var<N>, E, C> {
        Chan::from_raw(self.into_raw())
    }
}

impl<C: RawChan, E> Chan<Close, E, C> {
    pub async fn close(self) -> Result<(), Error> {
        self.0.close().await
    }
}

#[macro_export]
macro_rules! send {
    ( $c:ident, $msg:expr ) => {
        let $c = $c.send($msg).await?;
    };
}

#[macro_export]
macro_rules! recv {
    ( $c:ident, $msg:ident ) => {
        let ($msg, $c) = $c.recv().await?;
    };
}

#[macro_export]
macro_rules! close {
    ( $c:ident ) => {
        $c.close().await?;
    };
}
#[macro_export]
macro_rules! choose_offer {
    ( $choose:ident $offer:ident {
        $($p:ident ( $t:ty ) $o:ident),+
    } with $offerExt:ident) => {
        struct $choose {
        }

        enum $offer {
        }

        impl HasDual for $choose { type Dual = $offer; }

        impl HasDual for $offer { type Dual = $choose; }

        choose_offer!($choose; 0; $($p($t)),*); // call def choose

        choose_offer!($offer $offerExt; $($o < $t as HasDual>::Dual),*);
    };

    // def choose
    ($choose:ident; $e:expr ; ) => {};
    ($choose:ident; $e:expr ; $p:ident ($t:ty) $(, $ps:ident ($ts:ty) )* ) => {
        async fn $p<E, C: RawChan>(chan: Chan< $choose, E, C>) -> Result< Chan< $t, E, C>, Error>
        where
            <C as $crate::RawChan>::R: $crate::Repr<u8>,
        {
            let mut c = chan.into_raw();
            c.send(<C::R as Repr<u8>>::from($e)).await?;
            Ok(Chan::from_raw(c))
        }


        choose_offer!($choose; $e + 1; $($ps ($ts)),* );
    };

    // def offer

    (offer_body $ext:ident $t:ident $c:ident ; $e:expr ; ) => {
            Err(Error::ConvertErr)
    };
    (offer_body $ext:ident $t:ident $c:ident ; $e:expr ; $p:ident $(, $ps:ident)* ) => {
        if ($t == $e) {
            Ok($ext::$p(Chan::from_raw($c)))
        } else {
            choose_offer!(offer_body $ext $t $c ; $e + 1 ; $( $ps ),*)
        }
    };

    ($offer:ident $ext:ident; $($p:ident $t:ty),+) => {

        pub enum $ext<E, C: RawChan> {
            $( $p(Chan< $t, E, C>) ),*
        }

        impl<E, C: RawChan> OfferExt< $offer, E> for Chan< $offer, E, C>
        where
            C::R: Repr<u8>,
        {
            type C = C;
            type OfferChan = $ext<E, Self::C>;

            type OfferFuture = impl std::future::Future<Output = Result<Self::OfferChan, Error>> + 'static where Self:'static;
            fn offer(self) -> Self::OfferFuture {
                let mut c = self.into_raw();
                async move {
                    let r = c.recv().await.map_err(|_| Error::RecvErr)?;
                    let t: u8 = Repr::try_into(r).map_err(|_| Error::ConvertErr)?;
                    choose_offer!(offer_body $ext t c ; 0 ; $( $p ),* )
                }
            }
        }
    };
}

pub trait OfferExt<P: HasDual, E> {
    type C: RawChan;
    type OfferChan;
    type OfferFuture: Future<Output = Result<Self::OfferChan, Error>> + 'static
    where
        Self: 'static;
    fn offer(self) -> Self::OfferFuture;
}

pub mod mpsc;
