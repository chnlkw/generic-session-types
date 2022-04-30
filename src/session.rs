use std::marker::PhantomData;

pub struct Eps;
pub struct Recv<T, P>(PhantomData<(T, P)>);
pub struct Send<T, P>(PhantomData<(T, P)>);

pub trait HasDual {
    type Dual;
}

impl HasDual for Eps {
    type Dual = Eps;
}

impl<T, P: HasDual> HasDual for Recv<T, P> {
    type Dual = Send<T, P::Dual>;
}

impl<T, P: HasDual> HasDual for Send<T, P> {
    type Dual = Recv<T, P::Dual>;
}
