use super::AnyRef;

pub trait Item: 'static {
    type I<'a>: Iterator<Item = AnyRef>;

    /// All internal references
    fn references(&self) -> Self::I<'_>;
}
