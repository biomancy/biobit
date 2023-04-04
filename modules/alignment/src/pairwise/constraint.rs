#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Constraint {
    Equivalent,
    Gap,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ConstrainedPos {
    pub pos: usize,
    pub op: Constraint,
}