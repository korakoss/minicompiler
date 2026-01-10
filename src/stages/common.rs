
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FuncId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
pub struct BlockId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct CellId(pub usize);
