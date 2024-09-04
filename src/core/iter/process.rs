use super::*;

/// Abstraction over graph iteration
/// T - Item
/// IN - Input to node processor
/// OUT - Output from node processor
pub trait ProcessDAG<
    'a,
    C: AnyContainer + ?Sized,
    P: Permit,
    TP: 'a + TypePermit + Permits<T>,
    IN,
    T: DynItem + ?Sized,
    OUT,
>
{
    /// Makes one step in processing
    #[must_use]
    fn step<'b>(
        &mut self,
        access: Access<'b, C, P, TP, All>,
    ) -> Option<IterNode<'a, 'b, P, T, IN, OUT>>
    where
        'a: 'b;

    /// Recomputes order of groups.
    fn recompute_order<'b>(&mut self, access: Access<'b, C, P, TP, All>)
    where
        'a: 'b;
}

pub trait ProcessDAGGrouped<
    'a,
    C: AnyContainer + ?Sized,
    P: Permit,
    TP: 'a + TypePermit + Permits<T>,
    IN,
    T: DynItem + ?Sized,
    OUT,
>: ProcessDAG<'a, C, P, TP, IN, T, OUT>
{
    /// Adds group of roots and returns their index.
    fn add_group(&mut self, group: &[Key<Ref<'a>, T>]) -> usize;

    /// Group must exist
    fn add_to_group(&mut self, group: usize, root: Key<Ref<'a>, T>);

    /// Removes group.
    /// It's index may be reused.
    fn purge(&mut self, group: usize);
}

pub struct DAGProcess<
    'a,
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
> {
    core: DAGCore<'a, C, S, P, NI, NP, NO, EP, EI, O, I>,
    access: I::Paused,
}

impl<
        'a,
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
    > DAGProcess<'a, C, S, P, NI, NP, NO, EP, EI, O, I>
{
    pub(super) fn new(
        access: I::Paused,
        core: DAGCore<'a, C, S, P, NI, NP, NO, EP, EI, O, I>,
    ) -> Self {
        Self { core, access }
    }

    fn with_access<'b, TP: 'b + TypePermit + Permits<S::T>, R>(
        &mut self,
        access: Access<'b, I::C, I::R, TP, All>,
        f: impl FnOnce(&mut Self, &mut I::B<'b, TP>) -> R,
    ) -> R {
        let mut access = I::resume(access, std::mem::take(&mut self.access));
        let r = f(self, &mut access);
        self.access = access.pause();
        r
    }
}

impl<
        'a,
        S: Start<'a>,
        NI,
        NP: FnMut((Option<O::Key>, I::Group), &[NI], &mut Slot<I::R, S::T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<I::R, S::T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, S::T>)>,
        O: Order<NI, I::C, I::R, S::T, S::K>,
        I: IsolateTemplate<S::T, Group = S::K>,
        TP: 'a + TypePermit + Permits<S::T>,
    > ProcessDAG<'a, I::C, I::R, TP, NI, S::T, NO>
    for DAGProcess<'a, I::C, S, I::R, NI, NP, NO, EP, EI, O, I>
{
    fn step<'b>(
        &mut self,
        access: Access<'b, I::C, I::R, TP, All>,
    ) -> Option<IterNode<'a, 'b, I::R, S::T, NI, NO>>
    where
        'a: 'b,
    {
        match self.core.process_next() {
            Some(Ok(node)) => Some(node),
            Some(Err((key, group, inputs, next))) => {
                let keys = I::access_paused(&mut self.access, group);
                let mut access = access.keys_split_with(keys);
                Some(self.core.process(key, group, inputs, next, &mut access))
            }
            None => None,
        }
    }

    fn recompute_order<'b>(&mut self, access: Access<'b, I::C, I::R, TP, All>)
    where
        'a: 'b,
    {
        self.with_access(access, |s, access| s.core.recompute_order(access))
    }
}

impl<
        'a,
        T: DynItem + ?Sized,
        NI,
        NP: FnMut((Option<O::Key>, I::Group), &[NI], &mut Slot<I::R, T>) -> Option<NO> + 'a,
        NO,
        EP: FnMut(&NO, &mut Slot<I::R, T>) -> EI,
        EI: Iterator<Item = (NI, Key<Ref<'a>, T>)>,
        O: Order<NI, I::C, I::R, T, usize>,
        I: IsolateTemplate<T, Group = usize>,
        TP: 'a + TypePermit + Permits<T>,
    > ProcessDAGGrouped<'a, I::C, I::R, TP, NI, T, NO>
    for DAGProcess<'a, I::C, Subset<'a, T, usize>, I::R, NI, NP, NO, EP, EI, O, I>
{
    fn add_group(&mut self, group: &[Key<Ref<'a>, T>]) -> usize {
        let i = I::add_group_paused(&mut self.access);
        self.core.add_group(i, group.iter().copied());
        i
    }

    fn add_to_group(&mut self, group: usize, root: Key<Ref<'a>, T>) {
        self.core.add_to_group(group, root);
    }

    fn purge(&mut self, group: usize) {
        I::remove_group_paused(&mut self.access, group);
        self.core.purge(group);
    }
}
