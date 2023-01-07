use std::{sync::Mutex, collections::HashMap};

use crate::core::models::anchor::Anchor;

use super::anchor_repository::AnchorRepository;

pub struct InMemoryAnchorRepository {
    next_id: Mutex<u32>,
    stored: Mutex<HashMap<u32, Anchor>>
}

impl InMemoryAnchorRepository {
    pub fn new() -> Self {
        InMemoryAnchorRepository {
            next_id: Mutex::new(1),
            stored: Mutex::new(HashMap::new())
        }
    }
}

impl AnchorRepository for InMemoryAnchorRepository {

    fn store(&mut self, anchor: &Anchor) -> Result<Anchor, String> {
        let temp: Anchor;
        let stored = match anchor.id() {
            Some(_) => { // already stored, so don't need to set ID
                anchor
            }
            None => { // not stored yet, so insert into DB
                let mut mutex = self.next_id.lock().unwrap();
                temp = anchor.with_id(*mutex);
                *mutex += 1;
                &temp
            }
        };        
        self.stored.lock().unwrap().insert(stored.id().unwrap(), stored.clone());
        Ok(stored.clone())
    }

    fn get_by_id(&self, id: u32) -> Result<Option<Anchor>, String> {
        let binding = self.stored.lock().unwrap();
        let found = binding.get(&id);
        match found {
            Some(anchor) => Ok(Some((*anchor).clone())),
            None => Ok(None)
        }
    }

    fn remove_by_id(&mut self, id: u32) -> Result<bool, String> {
        let mut binding = self.stored.lock().unwrap();
        if binding.contains_key(&id) {
            binding.remove(&id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for InMemoryAnchorRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*; // import everything from outer module

    #[test]
    fn store_given_new_anchor_sets_id() -> Result<(), String> {
        let mut sut = InMemoryAnchorRepository::new();
        let new_anchor = Anchor::new("Foo Bar");

        let result = sut.store(&new_anchor)?;

        assert!(result.id().is_some());

        Ok(())
    }

    #[test]
    fn store_given_two_new_anchors_gives_different_ids() -> Result<(), String> {
        let mut sut = InMemoryAnchorRepository::new();
        let first_anchor = Anchor::new("Foo Bar");
        let second_anchor = Anchor::new("Baz Qux");

        let first_id = sut.store(&first_anchor)?.id().unwrap();
        let second_id = sut.store(&second_anchor)?.id().unwrap();

        assert_ne!(first_id, second_id);

        Ok(())
    }

    #[test]
    fn store_given_old_anchor_id_updates_the_stored_anchor() -> Result<(), String> {
        let old_anchor = Anchor::new("Foo Bar").with_id(1);
        let mut sut = InMemoryAnchorRepository::new();
        sut.store(&old_anchor)?;
        
        let new_anchor = old_anchor.with_name("Baz Qux");

        sut.store(&new_anchor)?;

        let all_stored = sut.stored.lock().unwrap();
        let stored = all_stored.get(&1);

        assert!(stored.is_some());
        assert_eq!(stored.unwrap().name(), new_anchor.name());

        Ok(())
    }

    #[test]
    fn get_by_id_given_invalid_id_returns_none() -> Result<(), String> {
        let sut = InMemoryAnchorRepository::new();

        let result = sut.get_by_id(0);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        Ok(())
    }

    #[test]
    fn get_by_id_given_valid_id_returns_anchor_with_that_id() -> Result<(), String> {
        let anchor = Anchor::new("Foo Bar").with_id(1);
        let mut sut = InMemoryAnchorRepository::new();
        sut.store(&anchor)?;

        let result = sut.get_by_id(1);

        assert!(result.is_ok());
        assert!(result?.unwrap().id().unwrap() == 1);

        Ok(())
    }

    #[test]
    fn remove_by_id_given_invalid_id_returns_false() -> Result<(), String> {
        let mut sut = InMemoryAnchorRepository::new();

        let result = sut.remove_by_id(0)?;

        assert!(!result);

        Ok(())
    }

    #[test]
    fn remove_by_id_given_valid_id_returns_true() -> Result<(), String> {
        let mut sut = InMemoryAnchorRepository::new();
        sut.store(&Anchor::new("Foo Bar").with_id(1))?;

        let result = sut.remove_by_id(1)?;

        assert!(result);

        Ok(())
    }

    #[test]
    fn remove_by_id_given_valid_id_deletes_from_repo() -> Result<(), String> {
        let mut sut = InMemoryAnchorRepository::new();
        sut.store(&Anchor::new("Foo Bar").with_id(1))?;

        sut.remove_by_id(1)?;

        assert!(!sut.stored.lock().unwrap().contains_key(&1));

        Ok(())
    }
}