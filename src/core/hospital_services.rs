use super::{hospital_models::Hospital, hospital_repository::{RepositoryError, HospitalRepository}};

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
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        Dummy {

        }

        impl HospitalRepository for Dummy {
            fn get_all_hospitals(&self) -> Result<Vec<Hospital>, RepositoryError>;
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
}