pub struct Depth;

pub struct Breadth;

pub struct Topological<F>(pub F);

pub struct TopologicalKey;

pub trait Order {}

impl Order for Depth {}

impl Order for Breadth {}

impl<F> Order for Topological<F> {}

impl Order for TopologicalKey {}
