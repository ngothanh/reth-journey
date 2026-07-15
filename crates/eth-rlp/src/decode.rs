pub trait Decodable {
    fn decode(buf: &mut &[u8]) -> Result<Self, Error>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Error {
    UnexpectedString,
    UnexpectedList,
    Overflow,
    InputTooShort,
    NonCanonical,
    Custom(&'static str),
}
