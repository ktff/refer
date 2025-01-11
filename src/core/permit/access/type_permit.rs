use super::*;

#[derive(Clone)]
pub struct Types(Vec<TypeId>);

impl Types {
    pub fn insert<T: Item>(&mut self) {
        self.0.push(TypeId::of::<T>());
    }

    pub fn try_insert<T: Item>(&mut self) -> bool {
        if self.contains::<T>() {
            false
        } else {
            self.insert::<T>();
            true
        }
    }

    pub fn contains<T: Item>(&self) -> bool {
        self.0.contains(&TypeId::of::<T>())
    }

    pub fn remove<T: Item>(&mut self) -> bool {
        let mut found = false;
        self.0.retain(|id| {
            let cmp = id == &TypeId::of::<T>();
            found |= cmp;
            !cmp
        });
        found
    }
}

impl Default for Types {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait TypePermit {
    type State: Clone;

    fn allowed<T: Item>(state: &Self::State) -> bool;
}

impl TypePermit for All {
    type State = ();

    fn allowed<T: Item>(_: &Self::State) -> bool {
        true
    }
}

impl<T: Item> TypePermit for T {
    type State = ();

    fn allowed<D: Item>(_: &Self::State) -> bool {
        TypeId::of::<T>() == TypeId::of::<D>()
    }
}

impl TypePermit for Types {
    type State = Types;

    fn allowed<T: Item>(state: &Self::State) -> bool {
        state.contains::<T>()
    }
}

impl<T: TypePermit> TypePermit for Not<T> {
    type State = T::State;

    fn allowed<D: Item>(state: &Self::State) -> bool {
        !T::allowed::<D>(state)
    }
}

pub trait RequiredTypePermit {
    type Permit;
}

impl<T: DynItem + ?Sized> RequiredTypePermit for T {
    default type Permit = All;
}

impl<T: Item> RequiredTypePermit for T {
    type Permit = T;
}

pub trait Permits<T: ?Sized>: TypePermit {}

impl<T: DynItem + ?Sized> Permits<T> for All {}

impl<T: Item> Permits<T> for T {}
