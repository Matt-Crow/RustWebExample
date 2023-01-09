use crate::core::{repositories::anchor_repository::AnchorRepository, models::anchor::Anchor};

pub struct AnchorService {
    /// "dyn AnchorRepository" means "something that implements the 
    /// AnchorRepository trait".
    /// Need to use a Box to store this on the heap, as the size of the field is
    /// not known at compile time.
    repository: Box<dyn AnchorRepository>
}

impl AnchorService {

    pub fn new<T>(repo: T) -> Self 
    where T: AnchorRepository + Sync + Send + 'static // repo must live no shorter than self
    {
        Self {
            repository: Box::new(repo)
        }
    }

    pub fn update(&mut self, anchor: &Anchor) -> Result<Anchor, String> {
        match anchor.id() {
            Some(id) => {
                let old = self.repository.get_by_id(id)?;
                match old {
                    Some(_) => self.repository.store(anchor),
                    None => Err(format!("No anchor exists with ID = {}", id))
                }
            }
            None => Err("Anchor has no ID".to_string())
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use mockall::mock;

    mock! {
        Dummy {

        }

        impl AnchorRepository for Dummy {
            fn store(&mut self, anchor: &Anchor) -> Result<Anchor, String>;
            fn get_by_id(&self, id: u32) -> Result<Option<Anchor>, String>;
        }
    }

    #[test]
    fn update_given_anchor_with_no_id_returns_error() {
        let mock = MockDummy::new();
        let mut sut = AnchorService::new(mock);

        let result = sut.update(&Anchor::new("Foo Bar"));

        assert!(result.is_err());
    }

    #[test]
    fn update_given_anchor_not_in_repository_returns_error() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_by_id()
            .returning(|_| Ok(None));
        let mut sut = AnchorService::new(mock);

        let result = sut.update(&Anchor::new("Foo Bar").with_id(1));

        assert!(result.is_err());
    }

    #[test]
    fn update_given_anchor_in_repository_forwards_to_store() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_by_id()
            .returning(|id| Ok(Some(Anchor::new("Foo Bar").with_id(id))));
        mock
            .expect_store()
            .returning(|a| Ok(a.clone()));
        let mut sut = AnchorService::new(mock);

        let result = sut.update(&Anchor::new("Baz Qux").with_id(1));

        assert!(result.is_ok());
    }
}