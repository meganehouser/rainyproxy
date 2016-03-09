use httparse_orig::Error;

pub enum ParseStatus<T> {
    Complete(T),
    InProgress,
    Err(Error),
}

impl<T> ParseStatus<T> {
    pub fn is_complete(&self) -> bool {
        match *self {
            ParseStatus::Complete(_) => true,
            _ => false,
        }
    }
}

pub trait Parsable {
    fn new() -> Self;
    fn parse(&mut self, buf: &[u8]) -> ParseStatus<usize>;
}

pub trait Sendable {
    fn to_bytes(&self) -> Vec<u8>;
}
