use super::*;

pub struct DAGIterator<
    'a: 'b,
    'b,
    C: AnyContainer + ?Sized,
    S: Start<'a>,
    P: Permit,
    NI,
    NP,
    NO,
    EP,
    EI,
    O: Order<NI, C, P, S::T, S::K>,
    I: IsolateTemplate<S::T>,
    A,
    TP: 'b,
> {
    core: DAGCore<'a, C, S, P, NI, NP, NO, EP, EI, O, I>,
    access: A,
    _access: PhantomData<&'b TP>,
}

impl<
        'a: 'b,
        'b,
        C: AnyContainer + ?Sized,
        S: Start<'a>,
        P: Permit,
        NI,
        NP,
        NO,
        EP,
        EI,
        O: Order<NI, C, P, S::T, S::K>,
        I: IsolateTemplate<S::T>,
        A,
        TP: 'b,
    > DAGIterator<'a, 'b, C, S, P, NI, NP, NO, EP, EI, O, I, A, TP>
{
    pub(super) fn new(access: A, core: DAGCore<'a, C, S, P, NI, NP, NO, EP, EI, O, I>) -> Self {
        Self {
            core,
            access,
            _access: PhantomData,
        }
    }
}

impl<
        'a: 'b,
        'b,
        T: DynItem + ?Sized,
        NI,
        NP,
        NO,
        EP,
        EI,
        O: Order<NI, I::C, I::R, T, I::Group>,
        I: IsolateTemplate<T, Group = ()>,
        TP: 'b + TypePermit + Permits<T>,
    >
    DAGIterator<'a, 'b, I::C, Subset<'a, T, ()>, I::R, NI, NP, NO, EP, EI, O, I, I::B<'b, TP>, TP>
{
    /// Group must exist
    pub fn add_root(&mut self, root: Key<Ref<'a>, T>) {
        self.core.add_root(root);
    }
}

impl<
        'a: 'b,
        'b,
        T: DynItem + ?Sized,
        NI,
        NP,
        NO,
        EP,
        EI,
        O: Order<NI, I::C, I::R, T, I::Group>,
        I: IsolateTemplate<T, Group = usize>,
        TP: 'b + TypePermit + Permits<T>,
    >
    DAGIterator<
        'a,
        'b,
        I::C,
        Subset<'a, T, usize>,
        I::R,
        NI,
        NP,
        NO,
        EP,
        EI,
        O,
        I,
        I::B<'b, TP>,
        TP,
    >
{
    /// Adds group of roots and returns their index.
    pub fn add_group(&mut self, group: impl IntoIterator<Item = Key<Ref<'a>, T>>) -> usize {
        let i = self.access.add_group();
        self.core.add_group(i, group);
        i
    }

    /// Group must exist
    pub fn add_to_group(&mut self, group: usize, root: Key<Ref<'a>, T>) {
        self.core.add_to_group(group, root);
    }

    /// Removes group.
    /// It's index may be reused.
    pub fn purge(&mut self, group: usize) {
        self.access.remove_group(group);
        self.core.purge(group);
    }
}

impl<
        'a,
        S: Start<'a>,
        NI,
        NP,
        NO,
        EP,
        EI,
        O: Order<NI, I::C, I::R, S::T, S::K>,
        I: IsolateTemplate<S::T, Group = S::K>,
    > DAGIterator<'a, 'a, I::C, S, I::R, NI, NP, NO, EP, EI, O, I, I::Paused, ()>
{
    pub fn resume<'b, TP: 'b + TypePermit + Permits<S::T>>(
        self,
        access: Access<'b, I::C, I::R, TP, All>,
    ) -> DAGIterator<'a, 'b, I::C, S, I::R, NI, NP, NO, EP, EI, O, I, I::B<'b, TP>, TP>
    where
        'a: 'b,
    {
        DAGIterator {
            core: self.core,
            access: I::resume(access, self.access),
            _access: PhantomData,
        }
    }
}

impl<
        'a: 'b,
        'b,
        S: Start<'a>,
        NI,
        NP: FnMut((Option<O::Key>, I::Group), &[NI], &mut Slot<I::R, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<I::R, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, I::C, I::R, S::T, S::K>,
        I: IsolateTemplate<S::T, Group = S::K>,
        TP: 'b + TypePermit + Permits<S::T>,
    > DAGIterator<'a, 'b, I::C, S, I::R, NI, NP, NO, EP, EI, O, I, I::B<'b, TP>, TP>
{
    pub fn pause(
        self,
    ) -> DAGIterator<'a, 'a, I::C, S, I::R, NI, NP, NO, EP, EI, O, I, I::Paused, ()> {
        DAGIterator {
            core: self.core,
            access: self.access.pause(),
            _access: PhantomData,
        }
    }

    /// Sets order for further processing.
    pub fn set_order(&mut self, order: O) {
        self.core.set_order(order);
    }

    /// Reorders queue according to new order.
    pub fn reorder(&mut self, order: O) {
        self.core.reorder(order, &mut self.access);
    }
}

impl<
        'a: 'b,
        'b,
        S: Start<'a>,
        NI,
        NP: FnMut((Option<O::Key>, I::Group), &[NI], &mut Slot<I::R, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<I::R, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, I::C, I::R, S::T, S::K>,
        I: IsolateTemplate<S::T, Group = S::K>,
        TP: 'b + TypePermit + Permits<S::T>,
    > Iterator for DAGIterator<'a, 'b, I::C, S, I::R, NI, NP, NO, EP, EI, O, I, I::B<'b, TP>, TP>
{
    type Item = IterNode<'a, 'b, I::R, S::T, NI, NO>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.core.process_next() {
            Some(Ok(node)) => Some(node),
            Some(Err((key, group, inputs, next))) => {
                Some(
                    self.core
                        .process(key, group, inputs, next, self.access.access_group(group)),
                )
            }
            None => None,
        }
    }
}
