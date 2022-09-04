use crate::core::*;

/// Adds references in item at key to shells.
/// item --ref--> others
///
/// Fails if any reference doesn't exist.
pub fn add_references<T: Item + ?Sized>(
    shells: &mut (impl AnyShells + ?Sized),
    key: Key<T>,
    item: &T,
) -> bool {
    // item --> others
    for (i, rf) in item.references(key.index()).enumerate() {
        if let Some(shell) = shells.get_shell_mut_any(rf.key()) {
            shell.add_from(key.into());
        } else {
            // Reference doesn't exist

            // Rollback and return error
            for rf in item.references(key.index()).take(i) {
                assert!(
                    shells
                        .get_shell_mut_any(rf.key())
                        .expect("Should exist")
                        .remove_from(key.into()),
                    "Should exist"
                );
            }

            return false;
        }
    }

    true
}

/// Updates diff of references between old and new items on key through shells.
///
/// Fails if reference is not valid.
pub fn update_diff<T: Item + ?Sized>(
    shells: &mut (impl AnyShells + ?Sized),
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
    for (i, cmp) in crate::util::merge(&old, &new).enumerate() {
        match cmp {
            (Some(_), Some(_)) | (None, None) => (),
            (Some(&rf), None) => {
                // We don't care so much about this reference missing.
                shells
                    .get_shell_mut_any(rf.key())
                    .map(|shell| shell.remove_from(key.into()));
            }
            (None, Some(rf)) => {
                if let Some(shell) = shells.get_shell_mut_any(rf.key()) {
                    shell.add_from(key.into());
                } else {
                    // Reference doesn't exist

                    // Rollback and return error
                    for cmp in crate::util::merge(&old, &new).take(i) {
                        match cmp {
                            (Some(_), Some(_)) | (None, None) => (),
                            (Some(rf), None) => {
                                shells
                                    .get_shell_mut_any(rf.key())
                                    .expect("Should exist")
                                    .add_from(key.into());
                            }
                            (None, Some(rf)) => {
                                assert!(
                                    shells
                                        .get_shell_mut_any(rf.key())
                                        .expect("Should exist")
                                        .remove_from(key.into()),
                                    "Should exist"
                                );
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

/// Removes references from item at key to others and
/// removes references from others to item.
///
/// Additional items that need to be removed are added to
/// remove list.
///
/// None if it doesn't exist
pub fn remove_references(
    coll: &mut (impl AnyAccess + ?Sized),
    key: AnyKey,
    remove: &mut Vec<AnyKey>,
) -> Option<()> {
    // remove item --> others
    let (item, shells) = coll.split_item_any(key)?;
    if let Some(references) = item.references_any(key.index()) {
        for rf in references {
            shells
                .get_shell_mut_any(rf.key())
                .map(|shell| shell.remove_from(key.into()));
        }
    }

    // item <-- others
    let (items, shell) = coll.split_shell_any(key).expect("Should exist");
    for rf in shell.from_any() {
        if !items
            .get_item_mut_any(rf)
            .map_or(true, |item| item.remove_reference(rf.index(), key.into()))
        {
            remove.push(rf);
        }
    }

    Some(())
}
