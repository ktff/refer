use crate::core::*;

/// Adds references in item at key to shells.
/// item --ref--> others
///
/// Fails if any reference doesn't exist.
pub fn add_references<T: Item, C: Container<T>>(
    mut shells: MutShells<C>,
    key: Key<T>,
    item: &T,
) -> bool {
    // item --> others
    for (i, rf) in item.references(key.index()).enumerate() {
        if let Some(mut shell_slot) = shells.borrow_mut().get(rf.key()) {
            shell_slot.add_from(key.into());
        } else {
            // Reference doesn't exist

            // Rollback and return error
            for rf in item.references(key.index()).take(i) {
                rf.disconnect(key.into(), shells.borrow_mut());
            }

            return false;
        }
    }

    true
}

/// Updates diff of references between old and new item on key through shells.
///
/// Fails if reference is not valid.
pub fn update_diff<T: Item>(
    mut shells: MutShells<impl Container<T>>,
    key: Key<T>,
    old: &T,
    new: &T,
) -> bool {
    // Preparation for diff computation
    let mut old = old.references(key.index()).collect::<Vec<_>>();
    let mut new = new.references(key.index()).collect::<Vec<_>>();
    old.sort();
    new.sort();

    // item --> others
    for (i, cmp) in crate::util::pair_up(&old, &new).enumerate() {
        match cmp {
            (Some(_), Some(_)) | (None, None) => (),
            (Some(&rf), None) => {
                // We don't care so much about this reference missing.
                shells
                    .borrow_mut()
                    .get(rf.key())
                    .map(|mut slot| slot.shell_mut().remove_from(key.into()));
            }
            (None, Some(rf)) => {
                if let Some(mut shell_slot) = shells.borrow_mut().get(rf.key()) {
                    shell_slot.add_from(key.into());
                } else {
                    // Reference doesn't exist

                    // Rollback and return error
                    for cmp in crate::util::pair_up(&old, &new).take(i) {
                        match cmp {
                            (Some(_), Some(_)) | (None, None) => (),
                            (Some(rf), None) => {
                                let _ = AnyRef::connect(key.into(), rf.key(), shells.borrow_mut());
                            }
                            (None, Some(rf)) => {
                                rf.disconnect(key.into(), shells.borrow_mut());
                            }
                        }
                    }

                    return false;
                }
            }
        }
    }

    true
}

/// Notifies items referencing this one that it was removed.
///
/// Additional items that need to be removed are added to
/// remove list.
///
/// None if it doesn't exist
pub fn notify_item_removed<C: AnyContainer>(
    Split {
        mut items,
        mut shells,
    }: Split<MutItems<C>, MutShells<C>>,
    key: AnyKey,
    remove: &mut Vec<AnyKey>,
) -> Option<()> {
    // remove item --> others
    let item_slot = items.borrow().get(key)?;
    if let Some(references) = item_slot.item().references_any(key.index()) {
        for rf in references {
            shells
                .borrow_mut()
                .get(rf.key())
                .map(|mut slot| slot.shell_mut().remove_from(key.into()));
        }
    }

    // item <-- others
    let shell_slot = shells.borrow_mut().get(key).expect("Should exist");
    for rf in shell_slot.shell().from_any() {
        if !items
            .borrow_mut()
            .get(rf)
            .map_or(true, |mut slot| slot.item_removed(key.into()))
        {
            remove.push(rf);
        }
    }

    Some(())
}
