use crate::core::{container::RegionContainer, container::TypeContainer, *};
use log::*;
use std::{
    any::TypeId,
    collections::{btree_map, hash_map::Entry, BTreeMap, HashMap, HashSet},
    ops::{Deref, DerefMut, RangeBounds},
};

pub struct ExclusivePermit<'a, C: AnyContainer + ?Sized> {
    permit: Permit<permit::Mut, permit::Slot>,
    container: &'a mut C,
}

impl<'a, C: AnyContainer + ?Sized> ExclusivePermit<'a, C> {
    pub fn new(container: &'a mut C) -> Self {
        Self {
            // SAFETY: Mut access is proof of exclusive access.
            permit: unsafe { Permit::<permit::Mut, permit::Slot>::new() },
            container,
        }
    }

    pub fn borrow_mut(&mut self) -> ExclusivePermit<'_, C> {
        ExclusivePermit {
            permit: self.permit.access(),
            container: self.container,
        }
    }

    pub fn access(&self) -> AnyPermit<permit::Ref, permit::Slot, C> {
        // SAFETY: We have at least read access for whole C.
        unsafe { AnyPermit::unsafe_new(self.permit.borrow(), &self) }
    }

    pub fn access_mut(&mut self) -> AnyPermit<permit::Mut, permit::Slot, C> {
        AnyPermit::new(self.container)
    }

    pub fn step<T: Item>(self) -> Option<ExclusivePermit<'a, C::Sub>>
    where
        C: TypeContainer<T>,
    {
        let Self { container, permit } = self;
        container
            .get_mut()
            .map(|container| ExclusivePermit { container, permit })
    }

    pub fn step_into(self, index: usize) -> Option<ExclusivePermit<'a, C::Sub>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        container
            .get_mut(index)
            .map(|container| ExclusivePermit { container, permit })
    }

    pub fn step_range(
        self,
        range: impl RangeBounds<usize>,
    ) -> Option<impl Iterator<Item = ExclusivePermit<'a, C::Sub>>>
    where
        C: RegionContainer,
    {
        let Self { container, permit } = self;
        Some(
            container
                .iter_mut(range)?
                .map(move |container| ExclusivePermit {
                    container,
                    permit: permit.access(),
                }),
        )
    }

    fn in_locality<T: Item>(&self, key: Key<T>, to: &impl LocalityPath) -> bool
    where
        C: Container<T>,
    {
        self.get_locality(to)
            .map(|locality| locality.prefix().contains_key(key))
            .unwrap_or(false)
    }

    fn clone_item<T: Item>(&mut self, key: Key<T>, to: &impl LocalityPath) -> Result<T>
    where
        C: Container<T>,
    {
        // Placement
        self.fill_locality(to);
        let to_locality = self.get_locality(to).expect("Should be valid locality");

        // Build Duplicate
        let duplicate = self
            .access()
            .slot(key)
            .get()?
            .duplicate(to_locality)
            .ok_or_else(|| ReferError::invalid_op(key, "duplicate"))?;

        // Finish
        Ok(duplicate)
    }

    /// Clones under given prefix.
    /// None if given type can't be placed under prefix.
    /// Panics if item can't be cloned, or item doesn't exist.
    fn clone_any_item(&mut self, original_key: AnyKey, locality: LocalityKey) -> AnyKey {
        // Build duplicate
        let original = self
            .access()
            .slot(original_key)
            .get_dyn()
            .expect("Should be valid key");
        let original_ty = original.item_type_id();
        let to_locality = self
            .get_locality_any(&locality, original_ty)
            .expect("Should be valid prefix");
        let duplicate = original
            .duplicate(to_locality)
            .ok_or_else(|| ReferError::invalid_op(original_key, "duplicate"))
            .expect("Must be able to duplicate");

        // Fill
        let duplicate_ty = duplicate.as_ref().type_id();
        match self.fill_slot_any(&locality, duplicate) {
            Ok(key) => key,
            Err(error) => {
                panic!("Failed to fill slot of ty:{:?} locality:{} with duplicate of ty:{:?}.Cause: {:?}",original_ty,locality,duplicate_ty,error);
            }
        }
    }

    /// Expects that slot exists
    fn drop_slot<T: Item>(&mut self, key: Key<T>)
    where
        C: Container<T>,
    {
        let (mut item, mut shell, locality) = self.unfill_slot(key).expect("Should be present");
        shell.clear(locality.allocator());
        item.displace(locality, None);
    }

    fn resolve_remove(&mut self, mut remove: Vec<AnyKey>) {
        // Recursive remove
        while let Some(other) = remove.pop() {
            // Detach

            // item --> others
            if self.detach_item(other).is_ok() {
                // item <-- others
                self.detach_shell(other, &mut remove).expect("Should exist");

                // Unfill, drop shell, and drop item
                self.unfill_slot_any(other.into());
            }
        }
    }
}

impl<'a, C: AnyContainer + ?Sized> ExclusivePermit<'a, C> {
    pub fn add<T: Item>(&mut self, locality: &impl LocalityPath, item: T) -> Result<Key<T>>
    where
        C: Container<T>,
    {
        let key = self
            .fill_slot(locality, item)
            .map_err(|_| ReferError::out_of_keys::<T>(locality))?;

        if let Err(error) = self.attach_item(key) {
            // Rollback filling the slot.
            self.drop_slot(key);

            return Err(error);
        }

        Ok(key)
    }

    /// Removes previous item and sets new one in his place while updating references accordingly.
    pub fn replace_item<T: Item>(&mut self, key: Key<T>, item: T) -> Result<T>
    where
        C: Container<T>,
    {
        let (items, shells) = self.access_mut().split();
        let mut slot = items.slot(key).get()?;

        // Update connections
        Self::replace_item_references(shells, slot.borrow(), &item)?;

        // Replace item
        slot.displace();
        Ok(std::mem::replace(slot.item_mut(), item))
    }

    /// Displaces item to locality and displaces shell.
    /// May have side effects that invalidate some Keys.
    pub fn displace<T: Item>(&mut self, from: Key<T>, to: &impl LocalityPath) -> Result<Key<T>>
    where
        C: Container<T>,
    {
        // Check if key is in to locality.
        if self.in_locality(from, to) {
            return Ok(from);
        }

        // Clone
        let item = self.clone_item(from, to)?;

        // Fill
        let to = self
            .fill_slot(to, item)
            .map_err(|_| ReferError::out_of_keys::<T>(to))?;

        // Move shell
        if let Err(error) = self.displace_shell(from, to) {
            // Rollback filling the slot.
            self.drop_slot(to);

            return Err(error);
        }

        // Rewire from -> other to to -> other
        self.replace_item_key(from, to);

        // Unfill and drop from
        self.drop_slot(from);

        // Success
        Ok(to)
    }

    /// Displaces item and removes shell.
    /// May have side effects that invalidate some Keys.
    pub fn displace_item<T: Item>(&mut self, from: Key<T>, to: &impl LocalityPath) -> Result<Key<T>>
    where
        C: Container<T>,
    {
        // Check if key is in to locality.
        if self.in_locality(from, to) {
            // Detach shell
            let mut remove = Vec::new();
            self.detach_shell(from.upcast(), &mut remove)?;
            self.resolve_remove(remove);

            Ok(from)
        } else {
            // Clone
            let item = self.clone_item(from, to)?;

            // Fill
            let to = self
                .fill_slot(to, item)
                .map_err(|_| ReferError::out_of_keys::<T>(to))?;

            // Rewire from -> other to to -> other
            self.replace_item_key(from, to);

            // Unfill and drop from
            self.drop_slot(from);

            Ok(to)
        }
    }

    /// Displaces shell from `from` to `to` so that all references are now pointing to `to`.
    /// May have side effects that invalidate some Keys.
    pub fn displace_shell<T: Item>(&mut self, from: Key<T>, to: Key<T>) -> Result<()>
    where
        C: Container<T>,
    {
        // Displace shell
        let mut moves = BTreeMap::new();
        self.displace_shell_references(from.upcast(), to.upcast(), &mut moves, |key| key == from)?;

        // Resolve other moves
        if !moves.is_empty() {
            let mut moved = HashSet::new();
            moved.insert(from.upcast());
            moved.insert(to.upcast());

            while let Some((from, (ty, prefix))) = moves.pop_first() {
                // Has been moved?
                if moved.insert(from) {
                    // Can anything satisfy it?
                    if let Some(prefix) = prefix {
                        // Does current place satisfy prefix?
                        if !prefix.contains_index(from.index()) {
                            // Move

                            // Placement
                            if let Some(locality) = self.fill_locality_any(&prefix, ty) {
                                // Clone & fill
                                let to = self.clone_any_item(from, locality);

                                // Move shell
                                self.displace_shell_references(from, to, &mut moves, |key| {
                                    moved.contains(&key)
                                })
                                .expect("Key should be valid");

                                // Rewire from -> other to to -> other
                                self.replace_any_item_key(from, to);

                                // Item Drop local
                                let mut slot = self
                                    .access_mut()
                                    .slot(from)
                                    .get_dyn()
                                    .expect("Key should be valid");
                                slot.displace();

                                // Unfill, drop shell, and drop item
                                self.unfill_slot_any(from);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // Duplicates item to locality and duplicates shell.
    pub fn duplicate<T: Item>(&mut self, key: Key<T>, to: &impl LocalityPath) -> Result<Key<T>>
    where
        C: Container<T>,
    {
        let to = self.duplicate_item(key, to)?;
        match self.duplicate_shell(key, to) {
            Ok(()) => Ok(to),
            Err(error) => {
                self.remove(to).expect("Should be valid key");
                Err(error)
            }
        }
    }

    /// Duplicates item to locality.
    pub fn duplicate_item<T: Item>(&mut self, key: Key<T>, to: &impl LocalityPath) -> Result<Key<T>>
    where
        C: Container<T>,
    {
        // Clone
        let item = self.clone_item(key, to)?;

        // Add
        self.add(to, item)
    }

    /// Duplicates shell from `from` to `to` so that all references to `from` now also point to `to`.
    pub fn duplicate_shell<T: Item>(&mut self, from: Key<T>, to: Key<T>) -> Result<()>
    where
        C: Container<T>,
    {
        let mut duplicates = HashMap::new();
        duplicates.insert(from.upcast(), Ok(to.upcast()));
        let attached = |other: AnyKey| other == to;
        let mut add = Vec::new();

        // Duplicate this shell
        self.duplicate_shell_references(
            from.upcast(),
            to.upcast(),
            &mut duplicates,
            &mut add,
            attached,
        )?;

        // Resolve other duplications
        while let Some((from, ty)) = add.pop() {
            let (prefix, replace) = match duplicates.remove(&from) {
                Some(Err(data)) => data,
                _ => panic!("Illegal state"),
            };

            // Duplicate item
            let locality = if let Some(locality) = self.fill_locality_any(&prefix, ty) {
                locality
            } else {
                self.fill_locality_any(&Path::default(), ty)
                    .expect("Should be able to clone item somewhere")
            };

            // Duplicate item
            let to = self.clone_any_item(from, locality);
            duplicates.insert(from, Ok(to));

            // Adjust references
            for replace in replace {
                let mut duplicate = self
                    .access_mut()
                    .slot(to)
                    .get_dyn()
                    .expect("Should be valid key");
                duplicate.replace_reference(replace.0, replace.1);
            }

            // Duplicate shell
            self.duplicate_shell_references(from, to, &mut duplicates, &mut add, attached)
                .expect("Should be valid key");
        }

        // Attach duplicate items
        for (_, duplicate_key) in duplicates {
            let duplicate_key =
                duplicate_key.expect("All duplicates should be created before attaching");
            if !attached(duplicate_key) {
                self.attach_any_item(duplicate_key)
                    .expect("Invalid Item references");
            }
        }

        Ok(())
    }

    /// T has it's local data dropped.
    /// May have side effects that invalidate some Keys.
    pub fn remove<T: Item>(&mut self, key: Key<T>) -> Result<T>
    where
        C: Container<T>,
    {
        // Detach

        // item --> others
        self.detach_item(key.upcast())?;

        // item <-- others
        let mut remove = Vec::new();
        self.detach_shell(key.upcast(), &mut remove)
            .expect("Should exist");

        // Unfill and drop shell
        let (item, _, _) = self.unfill_slot(key).expect("Should be present");

        // Propagate change
        self.resolve_remove(remove);

        Ok(item)
    }

    /// Adds references in item at key to shells.
    /// item --ref--> others
    ///
    /// Fails if any reference doesn't exist.
    /// On failure, rolls back all changes.
    ///
    /// Panics if keys don't exist.
    fn attach_item<T: Item>(&mut self, key: Key<T>) -> Result<()>
    where
        C: Container<T>,
    {
        let (items, shells) = self.access_mut().split();

        let item = items.slot(key).get().expect("Should be valid key");

        // item --> others
        Self::attach_item_loop(shells, key.upcast(), || item.iter_references())
    }

    /// Adds references in item at key to shells.
    /// item --ref--> others
    ///
    /// Fails if any reference doesn't exist.
    /// On failure, rolls back all changes.
    ///
    /// Panics if keys don't exist.
    fn attach_any_item(&mut self, key: AnyKey) -> Result<()> {
        let (items, shells) = self.access_mut().split();

        let item = items.slot(key).get_dyn().expect("Should be valid key");

        // item --> others
        Self::attach_item_loop(shells, key, || {
            item.iter_references_any().into_iter().flatten()
        })
    }

    fn attach_item_loop<I: Iterator<Item = AnyRef>>(
        mut shells: MutAnyShells<C>,
        key: AnyKey,
        iter: impl Fn() -> I,
    ) -> Result<()> {
        // item --> others
        for (i, rf) in iter().enumerate() {
            match shells.peek_dyn(rf.key()) {
                Ok(mut shell_slot) => shell_slot.shell_add(key),
                Err(error) => {
                    // Reference doesn't exist

                    // Rollback and return error
                    for rf in iter().take(i) {
                        shells
                            .disconnect_dyn(key, rf)
                            .map_err(|error| {
                                panic!(
                                    "Failed to disconnect {:?} -> {:?}, error: {}",
                                    key,
                                    rf.key(),
                                    error
                                );
                            })
                            .ok();
                    }

                    return Err(error);
                }
            }
        }

        Ok(())
    }

    /// Rewire from -> other to to -> other
    /// Panics if keys don't exist.
    fn replace_item_key<T: Item>(&mut self, from: Key<T>, to: Key<T>)
    where
        C: Container<T>,
    {
        let (items, mut shells) = self.access_mut().split();

        let other = items.slot(from).get().expect("Should be valid key");
        for other_rf in other.iter_references() {
            other_rf
                .get_dyn(shells.borrow_mut())
                .shell_replace(from, to);
        }
    }

    /// Rewire from -> other to to -> other
    /// Panics if keys don't exist.
    fn replace_any_item_key(&mut self, from: AnyKey, to: AnyKey) {
        let (items, mut shells) = self.access_mut().split();

        let other = items.slot(from).get_dyn().expect("Should be valid key");
        if let Some(references) = other.iter_references_any() {
            for other_rf in references {
                other_rf
                    .get_dyn(shells.borrow_mut())
                    .shell_replace(from, to);
            }
        };
    }

    /// Updates diff of references between old and new item on key through shells.
    ///
    /// Fails if reference is not valid.
    fn replace_item_references<T: Item>(
        mut shells: MutAnyShells<C>,
        slot: Slot<T, C::Shell, permit::Ref, permit::Item>,
        new: &T,
    ) -> Result<()>
    where
        C: Container<T>,
    {
        // Preparation for diff computation
        let key = slot.key();
        let mut old = slot.iter_references().collect::<Vec<_>>();
        let mut new = new.iter_references(slot.locality()).collect::<Vec<_>>();
        old.sort();
        new.sort();

        // item --> others
        for (i, cmp) in crate::util::pair_up(&old, &new).enumerate() {
            match cmp {
                (Some(_), Some(_)) | (None, None) => (),
                (Some(&rf), None) => {
                    // We don't care so much about this reference missing.
                    let _ = shells
                        .peek_dyn(rf.key())
                        .map(|mut slot| slot.shell_remove(key));
                }
                (None, Some(rf)) => {
                    match shells.peek_dyn(rf.key()) {
                        Ok(mut shell_slot) => shell_slot.shell_add(key),
                        Err(error) => {
                            // Rollback and return error
                            for cmp in crate::util::pair_up(&old, &new).take(i) {
                                match cmp {
                                    (Some(_), Some(_)) | (None, None) => (),
                                    (Some(rf), None) => {
                                        shells
                                            .connect_dyn(key.upcast(), rf.key())
                                            .map_err(|error| {
                                                panic!(
                                                    "Failed to connect {:?} -> {:?}, error: {}",
                                                    key,
                                                    rf.key(),
                                                    error
                                                )
                                            })
                                            .ok();
                                    }
                                    (None, Some(&rf)) => {
                                        shells
                                            .disconnect_dyn(key.upcast(), rf)
                                            .map_err(|error| {
                                                panic!(
                                                    "Failed to disconnect {:?} -> {:?}, error: {}",
                                                    key,
                                                    rf.key(),
                                                    error
                                                )
                                            })
                                            .ok();
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
    fn displace_shell_references(
        &mut self,
        from: AnyKey,
        to: AnyKey,
        moves: &mut BTreeMap<AnyKey, (TypeId, Option<Path>)>,
        moved: impl Fn(AnyKey) -> bool,
    ) -> Result<()> {
        let (mut items, shells) = self.access_mut().split();

        // Fetch shells
        let (mut from_shell, mut to_shell) = match shells.split_pair(from, to) {
            // From == to
            None => return Ok(()),
            Some((from, to)) => (from.get_dyn()?, to.get_dyn()?),
        };

        // Move references in items -> from_shell
        if let Some(references) = from_shell.iter_any() {
            for (count, other_rf) in references.dedup() {
                if moved(other_rf.key()) {
                    other_rf
                        .get_dyn(items.borrow_mut())
                        .replace_reference(from, to);
                } else {
                    let mut other = other_rf.get_dyn(items.borrow_mut());
                    if let Some(under_prefix) = other.displace_reference(from, to) {
                        // Register move
                        match moves.entry(other_rf.key()) {
                            btree_map::Entry::Occupied(mut entry) => {
                                let prefix = entry.get_mut();
                                prefix.1 = prefix
                                    .1
                                    .as_ref()
                                    .and_then(|&prefix| prefix.and(under_prefix));
                            }
                            btree_map::Entry::Vacant(entry) => {
                                entry.insert((other.item_type_id(), Some(under_prefix)));
                            }
                        }
                    }
                }

                to_shell.shell_add_many(other_rf.key(), count);
            }
        }

        //Finish
        from_shell.shell_clear();

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
    fn duplicate_shell_references(
        &mut self,
        from: AnyKey,
        to: AnyKey,
        duplicates: &mut HashMap<
            AnyKey,
            std::result::Result<AnyKey, (Path, Vec<(AnyKey, AnyKey)>)>,
        >,
        add_duplicate: &mut Vec<(AnyKey, TypeId)>,
        attached: impl Fn(AnyKey) -> bool,
    ) -> Result<()> {
        let (mut items, shells) = self.access_mut().split();

        // Fetch shells
        let (from_shell, mut to_shell) = match shells.split_pair(from, to) {
            // From == to
            None => return Ok(()),
            Some((from, to)) => (from.get_dyn()?, to.get_dyn()?),
        };

        // Duplicate references in items -> from_shell
        if let Some(references) = from_shell.iter_any() {
            for (count, other_ref) in references.dedup().filter(|(_, rf)| !attached(rf.key())) {
                // NOTE: Duplicate items aren't attached at this point
                let mut other = other_ref.get_dyn(items.borrow_mut());
                if let Some(under_prefix) = other.duplicate_reference(from, to) {
                    match duplicates.entry(other_ref.key()) {
                        Entry::Vacant(entry) => {
                            entry.insert(Err((under_prefix, vec![(from, to)])));
                            add_duplicate.push((other_ref.key(), other.item_type_id()));
                        }
                        Entry::Occupied(mut entry) => {
                            match entry.get_mut() {
                                // Duplicate was created
                                Ok(duplicate_key) => {
                                    // Duplicate references only duplicate
                                    let mut duplicate = items
                                        .peek_dyn(*duplicate_key)
                                        .expect("Should be valid key");
                                    duplicate.replace_reference(from, to);

                                    if attached(*duplicate_key) {
                                        // Duplicate is attached
                                        to_shell.shell_add_many(*duplicate_key, count);
                                    }
                                }
                                Err((prefix, vec)) => {
                                    vec.push((from, to));
                                    *prefix = prefix.or(under_prefix);
                                }
                            }
                        }
                    }
                } else {
                    // Original reference duplicate
                    to_shell.shell_add_many(other_ref.key(), count);

                    // If duplicate was created, duplicate it's references.
                    if let Some(Ok(&mut duplicate_key)) =
                        duplicates.get_mut(&other_ref.key()).map(|d| d.as_mut())
                    {
                        // Duplicate reference duplicate
                        let mut duplicate =
                            items.peek_dyn(duplicate_key).expect("Should be valid key");
                        assert!(
                            duplicate.duplicate_reference(from, to).is_none(),
                            "Should be able to duplicate reference"
                        );

                        if attached(duplicate_key) {
                            // Duplicate is attached
                            to_shell.shell_add_many(duplicate_key, count);
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
    fn detach_item(&mut self, key: AnyKey) -> Result<()> {
        let (mut items, mut shells) = self.access_mut().split();

        let mut item_slot = items.peek_dyn(key)?;
        // Disconnect from other shells
        if let Some(references) = item_slot.iter_references_any() {
            for rf in references {
                shells
                    .disconnect_dyn(key, rf)
                    .map_err(|error| {
                        warn!(
                            "Failed to disconnect {:?} -> {:?}, error: {}",
                            key,
                            rf.key(),
                            error
                        );
                    })
                    .ok();
            }
        }
        // Clear local data
        item_slot.displace();

        Ok(())
    }

    /// Detaches other items from shell.
    ///
    /// Items that need to be removed are added to remove list.
    ///
    /// Err if key doesn't exist.
    fn detach_shell(&mut self, key: AnyKey, remove: &mut Vec<AnyKey>) -> Result<()> {
        let (mut items, mut shells) = self.access_mut().split();

        let mut shell_slot = shells.peek_dyn(key)?;
        if let Some(references) = shell_slot.iter_any() {
            remove.extend(
                references
                    .dedup()
                    .map(|(_, other_rf)| other_rf.key())
                    .filter(|&other_key| match items.peek_dyn(other_key) {
                        Ok(mut other) => other.remove_reference(key),
                        Err(_) => false,
                    }),
            );
        }

        // Clear local data
        shell_slot.shell_clear();

        Ok(())
    }
}

impl<'a, C: AnyContainer + ?Sized> Deref for ExclusivePermit<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl<'a, C: AnyContainer + ?Sized> DerefMut for ExclusivePermit<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         container::{all::AllContainer, vec::VecContainerFamily},
//         item::{edge::Edge, vertice::Vertice},
//     };

//     #[test]
//     fn reference_add() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add_with(i, ()).unwrap())
//             .collect::<Vec<_>>();

//         let center = collection
//             .add_with(
//                 Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
//                 (),
//             )
//             .ok()
//             .unwrap();

//         for node in nodes {
//             assert_eq!(
//                 collection
//                     .get(node)
//                     .unwrap()
//                     .1
//                     .from::<Vertice<usize>>()
//                     .collect::<Vec<_>>(),
//                 vec![center]
//             );
//         }
//     }

//     #[test]
//     fn reference_add_abort() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add_with(i, ()).unwrap())
//             .collect::<Vec<_>>();

//         collection.remove(nodes[n - 1]).unwrap();

//         assert!(collection
//             .add_with(
//                 Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
//                 ()
//             )
//             .is_err());

//         for &node in nodes.iter().take(n - 1) {
//             assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
//         }
//     }

//     #[test]
//     fn reference_set() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add(i).unwrap())
//             .collect::<Vec<_>>();

//         let center = collection
//             .add_with(
//                 Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
//                 (),
//             )
//             .ok()
//             .unwrap();

//         collection
//             .set(
//                 center,
//                 Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
//             )
//             .ok()
//             .unwrap();

//         for &node in nodes.iter().take(5) {
//             assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
//         }

//         for &node in nodes.iter().skip(5) {
//             assert_eq!(
//                 collection
//                     .get(node)
//                     .unwrap()
//                     .1
//                     .from::<Vertice<usize>>()
//                     .collect::<Vec<_>>(),
//                 vec![center]
//             );
//         }
//     }

//     #[test]
//     fn reference_set_abort() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add(i).unwrap())
//             .collect::<Vec<_>>();

//         collection.remove(nodes[n - 1]).unwrap();

//         let center = collection
//             .add_with(
//                 Vertice::new(nodes.iter().take(5).copied().map(Ref::new).collect()),
//                 (),
//             )
//             .ok()
//             .unwrap();

//         assert!(collection
//             .add_with(
//                 Vertice::new(nodes.iter().skip(5).copied().map(Ref::new).collect()),
//                 ()
//             )
//             .is_err());

//         for &node in nodes.iter().take(5) {
//             assert_eq!(
//                 collection
//                     .get(node)
//                     .unwrap()
//                     .1
//                     .from::<Vertice<usize>>()
//                     .collect::<Vec<_>>(),
//                 vec![center]
//             );
//         }

//         for &node in nodes.iter().skip(5).take(4) {
//             assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
//         }
//     }

//     #[test]
//     fn reference_remove() {
//         let n = 10;
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let nodes = (0usize..n)
//             .map(|i| collection.add_with(i, ()).unwrap())
//             .collect::<Vec<_>>();

//         let center = collection
//             .add_with(
//                 Vertice::new(nodes.iter().copied().map(Ref::new).collect()),
//                 (),
//             )
//             .ok()
//             .unwrap();

//         let _ = collection.remove(center).unwrap();

//         for node in nodes {
//             assert_eq!(collection.get(node).unwrap().1.from_count(), 0);
//         }
//     }

//     #[test]
//     fn cascading_remove() {
//         let mut collection = Owned::new(AllContainer::<VecContainerFamily>::default());

//         let a = collection.add_with(0, ()).unwrap();
//         let b = collection.add_with(1, ()).unwrap();
//         let edge = collection
//             .add_with(Edge::new([Ref::new(a), Ref::new(b)]), ())
//             .unwrap();

//         assert_eq!(collection.get(a).unwrap().1.from_count(), 1);
//         assert_eq!(collection.get(b).unwrap().1.from_count(), 1);

//         let _ = collection.remove(a).unwrap();
//         assert!(collection.get(edge).is_none());
//         assert!(collection.get(a).is_none());
//         assert!(collection.get(b).unwrap().0 == (&1, &()));
//         assert_eq!(collection.get(b).unwrap().1.from_count(), 0);
//     }
// }
