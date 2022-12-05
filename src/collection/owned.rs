use crate::core::*;
use log::*;
use std::collections::{btree_map, hash_map::Entry, BTreeMap, HashMap, HashSet};

// TODO: Try to simplify algorithms

pub struct Owned<C: AnyContainer>(C);

impl<C: AnyContainer> Owned<C> {
    /// UNSAFE: Caller must ensure that complete ownership is transferred.
    pub unsafe fn new(c: C) -> Self {
        Self(c)
    }

    /// Temporary method to enable experimentation. Will be removed.
    pub fn inner(&self) -> &C {
        &self.0
    }

    /// Temporary method to enable experimentation. Will be removed.
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.0
    }

    fn in_locality<T: Item>(&self, key: Key<T>, to: T::Locality) -> bool
    where
        C: Model<T>,
    {
        self.0
            .locality_to_prefix(to)
            .map(|prefix| SubKey::from(key).of(prefix))
            .unwrap_or(false)
    }

    fn fill_item<T: Item>(
        &mut self,
        locality: T::Locality,
        item: T,
    ) -> Result<Key<T>, CollectionError>
    where
        C: Model<T>,
    {
        self.0
            .fill_slot(locality, item)
            .map_err(|_| CollectionError::out_of_keys::<T>(locality))
            .map(SubKey::into_key)
    }

    fn clone_item<T: Item>(&mut self, key: Key<T>, to: T::Locality) -> Result<T, CollectionError>
    where
        C: Model<T>,
    {
        // Placement
        let locality_prefix = self.0.select_locality(to);

        // Build Duplicate
        let to_context = self
            .0
            .context_any(locality_prefix)
            .expect("Should be valid prefix");
        let duplicate = self
            .get(key)?
            .duplicate(to_context)
            .ok_or_else(|| CollectionError::invalid_op(key, "duplicate"))?;

        // Finish
        let item = Box::into_inner(duplicate.downcast::<T>().expect("Wrong type"));
        Ok(item)
    }

    /// Clones under given prefix.
    /// Panics if item can't be cloned, or item doesn't exist.
    fn clone_any_item(&mut self, original_key: AnyKey, prefix: KeyPrefix) -> AnyKey {
        // Placement
        let locality_prefix = self.0.choose_locality(prefix);

        // Build duplicate
        let original = self.slot().get(original_key).expect("Should be valid key");
        let to_context = self
            .0
            .context_any(locality_prefix)
            .expect("Should be valid prefix");
        let duplicate = original
            .duplicate(to_context)
            .ok_or_else(|| CollectionError::invalid_op_any(original_key, "duplicate"))
            .expect("Must be able to duplicate");

        // Fill
        let duplicate_ty = duplicate.as_ref().type_id();
        match self
            .0
            .fill_slot_any(original_key.type_id(), locality_prefix, duplicate)
        {
            Ok(key) => key.into_key(),
            Err(error) => {
                panic!("Failed to fill slot of ty:{:?} locality:{} with duplicate of ty:{:?}.Cause: {:?}",original_key.type_id(),prefix,duplicate_ty,error);
            }
        }
    }

    /// Expects that slot exists
    fn drop_slot<T: Item>(&mut self, key: Key<T>)
    where
        C: Model<T>,
    {
        let (mut item, mut shell, context) =
            self.0.unfill_slot(key.into()).expect("Should be present");
        shell.dealloc(context.1);
        item.drop_local(ItemContext::<T>::new(context).upcast());
    }

    fn resolve_remove(&mut self, mut remove: Vec<AnyKey>) {
        // Recursive remove
        while let Some(other) = remove.pop() {
            // Detach

            // item --> others
            if detach_item(self.slot_mut().split_slots(), other).is_ok() {
                // item <-- others
                detach_shell(self.slot_mut().split_slots(), other, &mut remove)
                    .expect("Should exist");

                // Unfill, drop shell, and drop item
                self.0.unfill_slot_any(other.into());
            }
        }
    }
}

/// This is safe since Owned has full ownership of C.
unsafe impl<C: AnyContainer + 'static> Sync for Owned<C> {}

impl<C: AnyContainer> Access<C> for Owned<C> {
    fn slot(&self) -> AnyPermit<permit::Ref, permit::Slot, C> {
        // SAFETY: We have at least read access for whole C.
        unsafe { AnyPermit::new(&self.0) }
    }

    fn slot_mut(&mut self) -> AnyPermit<permit::Mut, permit::Slot, C> {
        // SAFETY: We have at least mut access for whole C.
        unsafe { AnyPermit::new(&self.0) }
    }
}

impl<T: Item, C: Model<T>> Collection<T> for Owned<C> {
    type Model = C;

    fn add(&mut self, locality: T::Locality, item: T) -> Result<Key<T>, CollectionError> {
        let key = self.fill_item(locality, item)?;

        if let Err(error) = attach_item(self.slot_mut().split_slots(), key) {
            // Rollback filling the slot.
            self.drop_slot(key);

            return Err(error);
        }

        Ok(key)
    }

    fn replace_item(&mut self, key: Key<T>, item: T) -> Result<T, CollectionError> {
        let (items, shells) = self.slot_mut().split_slots();
        let mut slot = items.of().get(key)?;

        // Update connections
        replace_item_references(shells, slot.borrow(), &item)?;

        // Replace item
        let context = slot.context();
        slot.drop_local(context.upcast());
        Ok(std::mem::replace(slot.item_mut(), item))
    }

    fn displace(&mut self, from: Key<T>, to: T::Locality) -> Result<Key<T>, CollectionError> {
        // Check if key is in to locality.
        if self.in_locality(from, to) {
            return Ok(from);
        }

        // Clone
        let item = self.clone_item(from, to)?;

        // Fill
        let to = self.fill_item(to, item)?;

        // Move shell
        if let Err(error) = self.displace_shell(from, to) {
            // Rollback filling the slot.
            self.drop_slot(to);

            return Err(error);
        }

        // Rewire from -> other to to -> other
        replace_item_key(self.slot_mut().split_slots(), from, to);

        // Unfill and drop from
        self.drop_slot(from);

        // Success
        Ok(to)
    }

    /// Moves item and removes shell.
    fn displace_item(&mut self, from: Key<T>, to: T::Locality) -> Result<Key<T>, CollectionError> {
        // Check if key is in to locality.
        if self.in_locality(from, to) {
            // Detach shell
            let mut remove = Vec::new();
            detach_shell(self.slot_mut().split_slots(), from.into(), &mut remove)?;
            self.resolve_remove(remove);

            Ok(from)
        } else {
            // Clone
            let item = self.clone_item(from, to)?;

            // Fill
            let to = self.fill_item(to, item)?;

            // Rewire from -> other to to -> other
            replace_item_key(self.slot_mut().split_slots(), from, to);

            // Unfill and drop from
            self.drop_slot(from);

            Ok(to)
        }
    }

    /// Moves shell from `from` to `to` so that all references are now pointing to `to`.
    /// May have side effects that invalidate some Keys.
    fn displace_shell(&mut self, from: Key<T>, to: Key<T>) -> Result<(), CollectionError> {
        // Displace shell
        let mut moves = BTreeMap::new();
        displace_shell_references(
            self.slot_mut().split_slots(),
            from.into(),
            to.into(),
            &mut moves,
            |key| key == from.into(),
        )?;

        // Resolve other moves
        if !moves.is_empty() {
            let mut moved = HashSet::new();
            moved.insert(from.into());
            moved.insert(to.into());

            while let Some((from, prefix)) = moves.pop_first() {
                // Has been moved?
                if moved.insert(from) {
                    // Does current place satisfy prefix?
                    if !prefix.prefix_of(from.index().0) {
                        // Move

                        // Clone & fill
                        let to = self.clone_any_item(from, prefix);

                        // Move shell
                        displace_shell_references(
                            self.slot_mut().split_slots(),
                            from,
                            to,
                            &mut moves,
                            |key| moved.contains(&key),
                        )
                        .expect("Key should be valid");

                        // Rewire from -> other to to -> other
                        replace_any_item_key(self.slot_mut().split_slots(), from, to);

                        // Item Drop local
                        let mut slot = self.slot_mut().get(from).expect("Key should be valid");
                        let context = slot.context();
                        slot.drop_local(context);

                        // Unfill, drop shell, and drop item
                        self.0.unfill_slot_any(from.into());
                    }
                }
            }
        }

        Ok(())
    }

    /// Duplicates item to locality.
    fn duplicate_item(&mut self, key: Key<T>, to: T::Locality) -> Result<Key<T>, CollectionError> {
        // Clone
        let item = self.clone_item(key, to)?;

        // Add
        self.add(to, item)
    }

    /// Duplicates shell from `from` to `to` so that all references to `from` now also point to `to`.
    fn duplicate_shell(&mut self, from: Key<T>, to: Key<T>) -> Result<(), CollectionError> {
        let mut duplicates = HashMap::new();
        duplicates.insert(from.upcast(), Ok(to.upcast()));
        let attached = |other: AnyKey| other == to.upcast();
        let mut add = Vec::new();

        // Duplicate this shell
        duplicate_shell_references(
            self.slot_mut().split_slots(),
            from.upcast(),
            to.upcast(),
            &mut duplicates,
            &mut add,
            attached,
        )?;

        // Resolve other duplications
        while let Some(from) = add.pop() {
            let (prefix, replace) = match duplicates.remove(&from) {
                Some(Err(data)) => data,
                _ => panic!("Illegal state"),
            };

            // Duplicate item
            let to = self.clone_any_item(from, prefix);
            duplicates.insert(from, Ok(to));

            // Adjust references
            for replace in replace {
                let mut duplicate = self.slot_mut().get(to).expect("Should be valid key");
                duplicate.replace_reference(replace.0, replace.1);
            }

            // Duplicate shell
            duplicate_shell_references(
                self.slot_mut().split_slots(),
                from,
                to,
                &mut duplicates,
                &mut add,
                attached,
            )
            .expect("Should be valid key");
        }

        // Attach duplicate items
        for (_, duplicate_key) in duplicates {
            let duplicate_key =
                duplicate_key.expect("All duplicates should be created before attaching");
            if !attached(duplicate_key) {
                attach_any_item(self.slot_mut().split_slots(), duplicate_key)
                    .expect("Invalid Item references");
            }
        }

        Ok(())
    }

    fn remove(&mut self, key: Key<T>) -> Result<T, CollectionError> {
        // Detach

        // item --> others
        detach_item(self.slot_mut().split_slots(), key.into())?;

        // item <-- others
        let mut remove = Vec::new();
        detach_shell(self.slot_mut().split_slots(), key.into(), &mut remove).expect("Should exist");

        // Unfill and drop shell
        let (item, _, _) = self.0.unfill_slot(key.into()).expect("Should be present");

        // Propagate change
        self.resolve_remove(remove);

        Ok(item)
    }
}

impl<C: AnyContainer> Drop for Owned<C> {
    fn drop(&mut self) {
        for ty in self.0.types() {
            if let Some(mut key) = self.0.first(ty) {
                loop {
                    // Drop slot
                    match self.slot_mut().get(key.into_key()) {
                        Ok(mut slot) => {
                            // Drop local
                            let context = slot.context();
                            slot.shell_mut().dealloc(context.allocator());
                            slot.item_mut().drop_local(context);

                            // Unfill
                            self.0.unfill_slot_any(key);
                        }
                        Err(error) => warn!("Invalid key: {}", error),
                    }

                    // Next
                    if let Some(next) = self.0.next(key) {
                        key = next;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

/// Adds references in item at key to shells.
/// item --ref--> others
///
/// Fails if any reference doesn't exist.
/// On failure, rolls back all changes.
///
/// Panics if keys don't exist.
fn attach_item<T: Item, C: Model<T>>(
    (items, shells): (MutAnyItems<C>, MutAnyShells<C>),
    key: Key<T>,
) -> Result<(), CollectionError> {
    let item = items.of().get(key).expect("Should be valid key");

    // item --> others
    attach_item_loop(shells, key.into(), || item.iter_references())
}

/// Adds references in item at key to shells.
/// item --ref--> others
///
/// Fails if any reference doesn't exist.
/// On failure, rolls back all changes.
///
/// Panics if keys don't exist.
fn attach_any_item<C: AnyContainer>(
    (items, shells): (MutAnyItems<C>, MutAnyShells<C>),
    key: AnyKey,
) -> Result<(), CollectionError> {
    let item = items.get(key).expect("Should be valid key");

    // item --> others
    attach_item_loop(shells, key, || {
        item.iter_references_any().into_iter().flatten()
    })
}

fn attach_item_loop<C: AnyContainer, I: Iterator<Item = AnyRef>>(
    mut shells: MutAnyShells<C>,
    key: AnyKey,
    iter: impl Fn() -> I,
) -> Result<(), CollectionError> {
    // item --> others
    for (i, rf) in iter().enumerate() {
        match shells.borrow_mut().get(rf.key()) {
            Ok(mut shell_slot) => shell_slot.add_from(key),
            Err(error) => {
                // Reference doesn't exist

                // Rollback and return error
                for rf in iter().take(i) {
                    rf.disconnect_from(key, shells.borrow_mut());
                }

                return Err(error);
            }
        }
    }

    Ok(())
}

/// Rewire from -> other to to -> other
/// Panics if keys don't exist.
fn replace_item_key<T: Item, C: Model<T>>(
    (items, mut shells): (MutAnyItems<C>, MutAnyShells<C>),
    from: Key<T>,
    to: Key<T>,
) {
    let other = items.of::<T>().get(from).expect("Should be valid key");
    for other_rf in other.iter_references() {
        other_rf
            .get(shells.borrow_mut())
            .replace(from.into(), to.index());
    }
}

/// Rewire from -> other to to -> other
/// Panics if keys don't exist.
fn replace_any_item_key<C: AnyContainer>(
    (items, mut shells): (MutAnyItems<C>, MutAnyShells<C>),
    from: AnyKey,
    to: AnyKey,
) {
    let other = items.get(from).expect("Should be valid key");
    if let Some(references) = other.iter_references_any() {
        for other_rf in references {
            other_rf.get(shells.borrow_mut()).replace(from, to.index());
        }
    };
}

/// Updates diff of references between old and new item on key through shells.
///
/// Fails if reference is not valid.
fn replace_item_references<T: Item, C: Model<T>>(
    mut shells: MutAnyShells<C>,
    slot: Slot<T, C::Shell, permit::Ref, permit::Item>,
    new: &T,
) -> Result<(), CollectionError> {
    // Preparation for diff computation
    let key = slot.key();
    let mut old = slot.iter_references().collect::<Vec<_>>();
    let mut new = new.iter_references(slot.context()).collect::<Vec<_>>();
    old.sort();
    new.sort();

    // item --> others
    for (i, cmp) in crate::util::pair_up(&old, &new).enumerate() {
        match cmp {
            (Some(_), Some(_)) | (None, None) => (),
            (Some(&rf), None) => {
                // We don't care so much about this reference missing.
                let _ = shells
                    .borrow_mut()
                    .get(rf.key())
                    .map(|mut slot| slot.shell_mut().remove_from(key.into()));
            }
            (None, Some(rf)) => {
                match shells.borrow_mut().get(rf.key()) {
                    Ok(mut shell_slot) => shell_slot.add_from(key.into()),
                    Err(error) => {
                        // Rollback and return error
                        for cmp in crate::util::pair_up(&old, &new).take(i) {
                            match cmp {
                                (Some(_), Some(_)) | (None, None) => (),
                                (Some(rf), None) => {
                                    rf.key().connect_from(key, shells.borrow_mut());
                                }
                                (None, Some(rf)) => {
                                    rf.disconnect_from(key.into(), shells.borrow_mut());
                                }
                            }
                        }

                        return Err(error);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Displaces references from `from` to `to` so that all references are now pointing to `to`.
/// Adds additional moves to `moves` if necessary.
/// Doesn't move moved.
///
/// Errors if `from` or `to` don't exist.
fn displace_shell_references<C: AnyContainer>(
    (mut items, shells): (MutAnyItems<C>, MutAnyShells<C>),
    from: AnyKey,
    to: AnyKey,
    moves: &mut BTreeMap<AnyKey, KeyPrefix>,
    moved: impl Fn(AnyKey) -> bool,
) -> Result<(), CollectionError> {
    // Fetch shells
    let (mut from_shell, mut to_shell) = match shells.get_pair(from, to)? {
        // From == to
        None => return Ok(()),
        Some(shells) => shells,
    };

    // Move references in items -> from_shell
    if let Some(references) = from_shell.iter_any() {
        for (count, other_rf) in references.dedup() {
            if moved(other_rf.key()) {
                other_rf
                    .get(items.borrow_mut())
                    .replace_reference(from.into(), to.index());
            } else {
                if let Some(under_prefix) = other_rf
                    .get(items.borrow_mut())
                    .displace_reference(from.into(), to.index())
                {
                    // Register move
                    match moves.entry(other_rf.key()) {
                        btree_map::Entry::Occupied(mut entry) => {
                            let prefix = entry.get_mut();
                            *prefix = prefix.intersect(under_prefix);
                        }
                        btree_map::Entry::Vacant(entry) => {
                            entry.insert(under_prefix);
                        }
                    }
                }
            }

            to_shell.add_from_count(other_rf.key(), count);
        }
    }

    //Finish
    let alloc = from_shell.alloc();
    from_shell.dealloc(alloc);

    Ok(())
}

/// Duplicates shell from `from` to `to` so that all references to `from` now also point to `to`.
///
/// Additional slots to duplicate are added to duplicates with replace (from,to) pairs and key prefix
/// under which to place the duplicate, and add_duplicate with from.
///
/// Fn attached should return true if provided item is an attached duplicate.
///
/// Err if from or to doesn't exist.
fn duplicate_shell_references<C: AnyContainer>(
    (mut items, shells): (MutAnyItems<C>, MutAnyShells<C>),
    from: AnyKey,
    to: AnyKey,
    duplicates: &mut HashMap<AnyKey, Result<AnyKey, (KeyPrefix, Vec<(AnyKey, Index)>)>>,
    add_duplicate: &mut Vec<AnyKey>,
    attached: impl Fn(AnyKey) -> bool,
) -> Result<(), CollectionError> {
    // Fetch shells
    let (from_shell, mut to_shell) = match shells.get_pair(from, to)? {
        // From == to
        None => return Ok(()),
        Some(shells) => shells,
    };

    // Duplicate references in items -> from_shell
    if let Some(references) = from_shell.iter_any() {
        for (count, other_ref) in references.dedup().filter(|(_, rf)| !attached(rf.key())) {
            // NOTE: Duplicate items aren't attached at this point
            let mut other = other_ref.get(items.borrow_mut());
            if let Some(under_prefix) = other.duplicate_reference(from, to.index()) {
                match duplicates.entry(other_ref.key()) {
                    Entry::Vacant(entry) => {
                        entry.insert(Err((under_prefix, vec![(from, to.index())])));
                        add_duplicate.push(other_ref.key());
                    }
                    Entry::Occupied(mut entry) => {
                        match entry.get_mut() {
                            // Duplicate was created
                            Ok(duplicate_key) => {
                                // Duplicate references only duplicate
                                let mut duplicate = items
                                    .borrow_mut()
                                    .get(*duplicate_key)
                                    .expect("Should be valid key");
                                duplicate.replace_reference(from, to.index());

                                if attached(*duplicate_key) {
                                    // Duplicate is attached
                                    to_shell.add_from_count(*duplicate_key, count);
                                }
                            }
                            Err((prefix, vec)) => {
                                vec.push((from, to.index()));
                                *prefix = prefix.intersect(under_prefix);
                            }
                        }
                    }
                }
            } else {
                // Original reference duplicate
                to_shell.add_from_count(other_ref.key(), count);

                // If duplicate was created, duplicate it's references.
                if let Some(Ok(&mut duplicate_key)) =
                    duplicates.get_mut(&other_ref.key()).map(|d| d.as_mut())
                {
                    // Duplicate reference duplicate
                    let mut duplicate = items
                        .borrow_mut()
                        .get(duplicate_key)
                        .expect("Should be valid key");
                    assert!(
                        duplicate.duplicate_reference(from, to.index()).is_none(),
                        "Should be able to duplicate reference"
                    );

                    if attached(duplicate_key) {
                        // Duplicate is attached
                        to_shell.add_from_count(duplicate_key, count);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Detaches item from other shells.
///
/// Err if key doesn't exist.
fn detach_item<C: AnyContainer>(
    (mut items, mut shells): (MutAnyItems<C>, MutAnyShells<C>),
    key: AnyKey,
) -> Result<(), CollectionError> {
    let mut item_slot = items.borrow_mut().get(key)?;
    if let Some(references) = item_slot.iter_references_any() {
        for rf in references {
            rf.disconnect_from(key, shells.borrow_mut());
        }
    }
    // Clear local data
    let context = item_slot.context();
    item_slot.drop_local(context);

    Ok(())
}

/// Detaches other items from shell.
///
/// Items that need to be removed are added to remove list.
///
/// Err if key doesn't exist.
fn detach_shell<C: AnyContainer>(
    (mut items, mut shells): (MutAnyItems<C>, MutAnyShells<C>),
    key: AnyKey,
    remove: &mut Vec<AnyKey>,
) -> Result<(), CollectionError> {
    let mut shell_slot = shells.borrow_mut().get(key)?;
    if let Some(references) = shell_slot.shell().iter_any() {
        for (_, other_rf) in references.dedup() {
            if let Ok(mut other) = items.borrow_mut().get(other_rf.key()) {
                let context = other.context();
                if other.remove_reference(context, key) {
                    remove.push(other_rf.key());
                }
            }
        }
    }
    // Clear local data
    let alloc = shell_slot.alloc();
    shell_slot.dealloc(alloc);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        container::{all::AllContainer, vec::VecContainerFamily},
        item::{edge::Edge, vertice::Vertice},
    };

    #[test]
    fn reference_add() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        for node in nodes {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }
    }

    #[test]
    fn reference_add_abort() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        collection.remove(nodes[n - 1]).unwrap();

        assert!(collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                ()
            )
            .is_err());

        for &node in nodes.iter().take(n - 1) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn reference_set() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add(i).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        collection
            .set(
                center,
                Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
            )
            .ok()
            .unwrap();

        for &node in nodes.iter().take(5) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }

        for &node in nodes.iter().skip(5) {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }
    }

    #[test]
    fn reference_set_abort() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add(i).unwrap())
            .collect::<Vec<_>>();

        collection.remove(nodes[n - 1]).unwrap();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        assert!(collection
            .add_with(
                Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
                ()
            )
            .is_err());

        for &node in nodes.iter().take(5) {
            assert_eq!(
                collection
                    .get(node)
                    .unwrap()
                    .1
                    .from::<Vertice<usize>>()
                    .collect::<Vec<_>>(),
                vec![center]
            );
        }

        for &node in nodes.iter().skip(5).take(4) {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn reference_remove() {
        let n = 10;
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let nodes = (0usize..n)
            .map(|i| collection.add_with(i, ()).unwrap())
            .collect::<Vec<_>>();

        let center = collection
            .add_with(
                Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
                (),
            )
            .ok()
            .unwrap();

        let _ = collection.remove(center).unwrap();

        for node in nodes {
            assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
        }
    }

    #[test]
    fn cascading_remove() {
        let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

        let a = collection.add_with(0, ()).unwrap();
        let b = collection.add_with(1, ()).unwrap();
        let edge = collection
            .add_with(Edge::new([Ref::new(a), Ref::new(b)]), ())
            .unwrap();

        assert_eq!(collection.get(a).unwrap().1.from_count(), 1);
        assert_eq!(collection.get(b).unwrap().1.from_count(), 1);

        let _ = collection.remove(a).unwrap();
        assert!(collection.get(edge).is_none());
        assert!(collection.get(a).is_none());
        assert!(collection.get(b).unwrap().0 == (&1, &()));
        assert_eq!(collection.get(b).unwrap().1.from_count(), 0);
    }
}
