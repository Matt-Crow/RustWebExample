use std::marker::PhantomData;


pub trait Entity {
    fn get_table_name(&self) -> String;
}

pub enum MSSQLError {
    Other(String),
    NotImplemented
}

impl MSSQLError {
    fn other(message: &str) -> Self {
        Self::Other(String::from(message))
    }

    fn not_implemented() -> Self {
        Self::NotImplemented
    }
}

pub struct MSSQLRepository<T> 
where T: Entity {
    entity_type: PhantomData<T>
}

impl<T> MSSQLRepository<T> 
where T: Entity {

    async fn insert(object: T) -> Result<T, MSSQLError> {
        let q = format!("
            INSERT INTO {0} {1}
                 VALUES 
                    {2}
            ;
        ", object.get_table_name(), "(a, b, c)", "(1, 2, 3)");
        Err(MSSQLError::NotImplemented)
    }
}