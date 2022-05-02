use std::marker::PhantomData;

pub struct Close;
pub struct Recv<T, P>(PhantomData<(T, P)>);
pub struct Send<T, P>(PhantomData<(T, P)>);
pub struct Choose<P, Q>(PhantomData<(P, Q)>);
pub struct Offer<P, Q>(PhantomData<(P, Q)>);

pub trait HasDual {
    type Dual: HasDual;
}

impl HasDual for Close {
    type Dual = Close;
}

impl<T, P: HasDual> HasDual for Recv<T, P> {
    type Dual = Send<T, P::Dual>;
}

impl<T, P: HasDual> HasDual for Send<T, P> {
    type Dual = Recv<T, P::Dual>;
}

impl<P: HasDual, Q: HasDual> HasDual for Choose<P, Q> {
    type Dual = Offer<P::Dual, Q::Dual>;
}

impl<P: HasDual, Q: HasDual> HasDual for Offer<P, Q> {
    type Dual = Choose<P::Dual, Q::Dual>;
}
