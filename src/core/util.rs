use super::*;

/// Adds references in item at key to shells.
/// item --ref--> others
///
/// Fails if any reference doesn't exist.
pub fn add_references<T: Item + ?Sized>(
    shells: &mut (impl AnyShellCollection + ?Sized),
    key: Key<T>,
    item: &T,
) -> bool {
    // item --> others
    for (i, rf) in item.references(key.index()).enumerate() {
        if !shells.add_from(rf.key(), key.into()) {
            // Reference doesn't exist

            // Rollback and return error
            for rf in item.references(key.index()).take(i) {
                assert!(shells.remove_from(rf.key(), key.into()), "Should exist");
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
    shells: &mut (impl AnyShellCollection + ?Sized),
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
                let _ = shells.remove_from(rf.key(), key.into());
            }
            (None, Some(rf)) => {
                if !shells.add_from(rf.key(), key.into()) {
                    // Reference doesn't exist

                    // Rollback and return error
                    for cmp in crate::util::merge(&old, &new).take(i) {
                        match cmp {
                            (Some(_), Some(_)) | (None, None) => (),
                            (Some(rf), None) => {
                                assert!(shells.add_from(rf.key(), key.into()), "Should exist");
                            }
                            (None, Some(rf)) => {
                                assert!(shells.remove_from(rf.key(), key.into()), "Should exist");
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
    coll: &mut (impl AnyCollection + ?Sized),
    key: AnyKey,
    remove: &mut Vec<AnyKey>,
) -> Option<()> {
    let (items, shells) = coll.split_any_mut();

    // remove item --> others
    for rf in items.references(key)? {
        let _ = shells.remove_from(rf.key(), key.into());
    }

    // item <-- others
    for rf in shells.from(key).expect("Should exist") {
        if !items.remove_reference(rf, key.into()) {
            remove.push(rf);
        }
    }

    Some(())
}
