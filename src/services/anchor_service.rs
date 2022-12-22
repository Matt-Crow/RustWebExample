use crate::{repositories::anchor_repository::AnchorRepository, models::anchor::Anchor};

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

    pub fn create(&mut self, anchor: Anchor) -> Result<Anchor, String> {
        match anchor.id() {
            Some(_) => Err("Cannot create Anchor who already has an ID".to_string()),
            None => self.repository.store(&anchor)
        }
    }

    pub fn update(&mut self, anchor: Anchor) -> Result<Anchor, String> {
        match anchor.id() {
            Some(id) => {
                let old = self.repository.get_by_id(id)?;
                match old {
                    Some(_) => self.repository.store(&anchor),
                    None => Err(format!("No anchor exists with ID = {}", id))
                }
            }
            None => Err("Anchor has no ID".to_string())
        }
    }

    pub fn get_all(&self) -> Result<Vec<Anchor>, String> {
        self.repository.get_all()
    }

    pub fn get_by_id(&self, id: u32) -> Result<Option<Anchor>, String> {
        self.repository.get_by_id(id)
    }

    pub fn delete_anchor(&mut self, id: u32) -> Result<(), String> {
        let was_removed = self.repository.remove_by_id(id)?;

        if was_removed {
            Ok(())
        } else {
            Err(format!("No anchor exists with ID = {}", id))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::models::anchor::Anchor;

    use super::*;

    use mockall::mock;

    mock! {
        Dummy {

        }

        impl AnchorRepository for Dummy {
            fn store(&mut self, anchor: &Anchor) -> Result<Anchor, String>;
            fn get_all(&self) -> Result<Vec<Anchor>, String>;
            fn get_by_id(&self, id: u32) -> Result<Option<Anchor>, String>;
            fn remove_by_id(&mut self, id: u32) -> Result<bool, String>;
        }
    }

    #[test]
    fn create_given_anchor_with_id_returns_error() {
        let mock = MockDummy::new();
        let mut sut = AnchorService::new(mock);

        let result = sut.create(Anchor::new("Foo Bar").with_id(1));

        assert!(result.is_err());
    }

    #[test]
    fn create_given_anchor_without_id_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_store()
            .returning(|x| Ok(x.clone()));
        let mut sut = AnchorService::new(mock);

        let result = sut.create(Anchor::new("Foo Bar"));

        assert!(result.is_ok());
    }

    #[test]
    fn update_given_anchor_with_no_id_returns_error() {
        let mock = MockDummy::new();
        let mut sut = AnchorService::new(mock);

        let result = sut.update(Anchor::new("Foo Bar"));

        assert!(result.is_err());
    }

    #[test]
    fn update_given_anchor_not_in_repository_returns_error() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_by_id()
            .returning(|_| Ok(None));
        let mut sut = AnchorService::new(mock);

        let result = sut.update(Anchor::new("Foo Bar").with_id(1));

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

        let result = sut.update(Anchor::new("Baz Qux").with_id(1));

        assert!(result.is_ok());
    }

    #[test]
    fn get_all_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_all()
            .once()
            .returning(|| Ok(Vec::new()));
        let sut = AnchorService::new(mock);

        let result =sut.get_all();

        assert!(result.is_ok());
    }

    #[test]
    fn get_by_id_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_by_id()
            .once()
            .returning(|id| Ok(Some(Anchor::new("Foo Bar").with_id(id))));
        let sut = AnchorService::new(mock);

        let result = sut.get_by_id(1);

        assert!(result.is_ok());
    }

    #[test]
    fn delete_given_an_invalid_id_returns_error() {
        let mut mock = MockDummy::new();
        mock
            .expect_remove_by_id()
            .returning(|_| Ok(false));
        let mut sut = AnchorService::new(mock);

        let result = sut.delete_anchor(1);

        assert!(result.is_err());
    }

    #[test]
    fn delete_given_a_valid_id_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_remove_by_id()
            .returning(|_| Ok(true));
        let mut sut = AnchorService::new(mock);

        let result = sut.delete_anchor(1);

        assert!(result.is_ok());
    }
}