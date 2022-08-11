use super::AnyCollection;

/// Up - self
/// Down - self.down()
pub trait LayerRef {
    type Down: ?Sized;

    fn down(&self) -> &Self::Down;
}

impl<C: AnyCollection + ?Sized> LayerRef for C {
    type Down = C;
    /// Careful with using down since keys probably need to be adjusted for it.
    fn down(&self) -> &Self::Down {
        self
    }
}

pub trait LayerMut: LayerRef {
    fn down_mut(&mut self) -> &mut Self::Down;
}

impl<C: LayerRef<Down = C> + ?Sized> LayerMut for C {
    fn down_mut(&mut self) -> &mut Self::Down {
        self
    }
}
