use std::hash::Hash; 

pub trait Id: Copy + Eq + Hash {
    fn from_raw(id: usize) -> Self;
    fn raw(self) -> usize;
}

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(usize);

        impl Id for $name {
            fn from_raw(id: usize) -> Self { Self(id) }
            fn raw(self) -> usize { self.0 }
        }
    };
}

define_id!(FuncId);
define_id!(BlockId);
define_id!(CellId);
define_id!(ChunkId);


pub struct IdFactory<I: Id> {
    counter: usize,
    _marker: std::marker::PhantomData<I>,
}

impl<I: Id> IdFactory<I> {

    pub fn new() -> Self {
        Self{ counter: 0, _marker: std::marker::PhantomData}
    }

    pub fn new_from(start: usize) -> Self {
        Self{ counter: start, _marker: std::marker::PhantomData}
    }
    
    pub fn next_id(&mut self) -> I {
        let id = I::from_raw(self.counter);
        self.counter += 1;
        id
    }
}
