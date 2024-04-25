pub struct Depth;

pub struct Breadth;

pub struct Topological<F, T>(pub F, pub OrderJoin<T>);

pub struct TopologicalKey;

pub trait Order {}

impl Order for Depth {}

impl Order for Breadth {}

impl<F, T> Order for Topological<F, T> {}

impl Order for TopologicalKey {}

/// Joins multiple order values into one.
pub enum OrderJoin<T> {
    /// Returned value is stable and does not change.
    Stable,
    /// Join value is minimal.
    Min,
    /// Join value is maximal.
    Max,
    /// Custom join function.
    Custom(fn(T, T) -> T),
}
