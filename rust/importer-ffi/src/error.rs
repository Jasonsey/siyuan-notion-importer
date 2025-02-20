#[derive(Debug, thiserror::Error)]
#[error("{e:?}")] // default message is from anyhow.
pub struct MyError {
    e: anyhow::Error,
}

impl MyError {
    pub fn message(&self) -> String { self.to_string() }
}

impl From<anyhow::Error> for MyError {
    fn from(e: anyhow::Error) -> Self {
        Self { e }
    }
}

pub(crate) type MyResult<T> = Result<T, MyError>;