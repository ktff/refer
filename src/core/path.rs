use super::AnyCollection;

/// A path from top to bottom.
/// Fetching bottom can cost.
pub trait PathRef<'a>: 'a {
    type Top: AnyCollection + ?Sized + 'a;
    type Bottom: AnyCollection + ?Sized + 'a;

    fn top(&self) -> &Self::Top;

    fn bottom(&self) -> &Self::Bottom;
}

impl<'a, C: AnyCollection + ?Sized + 'a> PathRef<'a> for &'a C {
    type Top = C;
    type Bottom = C;

    fn top(&self) -> &C {
        self
    }

    fn bottom(&self) -> &C {
        self
    }
}

impl<'a, C: AnyCollection + ?Sized + 'a> PathRef<'a> for &'a mut C {
    type Top = C;
    type Bottom = C;

    fn top(&self) -> &C {
        self
    }

    fn bottom(&self) -> &C {
        self
    }
}

pub trait PathMut<'a>: PathRef<'a> {
    fn top_mut(&mut self) -> &mut Self::Top;

    fn bottom_mut(&mut self) -> &mut Self::Bottom;
}

impl<'a, C: AnyCollection + ?Sized + 'a> PathMut<'a> for &'a mut C
where
    Self: PathRef<'a, Top = C, Bottom = C>,
{
    fn top_mut(&mut self) -> &mut C {
        self
    }

    fn bottom_mut(&mut self) -> &mut C {
        self
    }
}
