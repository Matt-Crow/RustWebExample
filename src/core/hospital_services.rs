use super::{hospital_models::Hospital, hospital_repository::{RepositoryError, HospitalRepository, By}};

pub struct HospitalService {
    repository: Box<dyn HospitalRepository + 'static>
}

impl HospitalService {
    pub fn new(repository: impl HospitalRepository + 'static) -> Self {
        Self {
            repository: Box::new(repository)
        }
    }

    pub fn get_all_hospitals(&self) -> Result<Vec<Hospital>, RepositoryError> {
        self.repository.get_all_hospitals()
    }

    pub fn get_hospital_by_name(&self, name: &str) -> Result<Option<Hospital>, RepositoryError> {
        self.repository.get_hospital(&By::Name(name.to_owned()))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::core::hospital_repository::By;

    use super::*;
    use mockall::mock;

    mock! {
        Dummy {

        }

        impl HospitalRepository for Dummy {
            fn get_all_hospitals(&self) -> Result<Vec<Hospital>, RepositoryError>;
            fn get_hospital(&self, by: &By) -> Result<Option<Hospital>, RepositoryError>;
        }
    }

    #[test]
    fn get_all_hospitals_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_all_hospitals()
            .once()
            .returning(|| Ok(Vec::new()));
        let sut = HospitalService::new(mock);

        let result = sut.get_all_hospitals();

        assert!(result.is_ok());
    }

    #[test]
    fn get_hospital_by_name_forwards_to_repository() {
        let mut mock = MockDummy::new();
        mock
            .expect_get_hospital()
            .once()
            .returning(|_by| Ok(None));
        let sut = HospitalService::new(mock);

        let result = sut.get_hospital_by_name("Foo");

        assert!(result.is_ok());
    }
}