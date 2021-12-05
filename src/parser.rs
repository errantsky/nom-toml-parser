use nom::{Err, IResult};
use nom::character::complete::{line_ending, space0};
use nom::error::{ErrorKind, ErrorKind::CrLf, ParseError};

enum TomlValue {
    Str(String),
    Integer(usize),
    Float(f64),
    Boolean(bool),
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
    Array,
    InlineTable,
}

// whitespace
fn whitespace<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    space0(input)
}

// newline
fn newline<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    line_ending(input)
}

fn comment() {}

fn integer() {}

fn float() {}

fn string() {}

// key value pair
fn key_val_pair() {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace::<(&str, ErrorKind)>(" \tSomeText"), Ok(("SomeText", " \t")));
        assert_eq!(whitespace::<(&str, ErrorKind)>("Test"), Ok(("Test", "")));
    }

    #[test]
    fn test_newline() {
        assert_eq!(newline::<(&str, ErrorKind)>("\nTest"), Ok(("Test", "\n")));
        assert_eq!(newline::<(&str, ErrorKind)>("\r\nTest"), Ok(("Test", "\r\n")));
        assert_eq!(newline::<(&str, ErrorKind)>("\rTest"), Err(Err::Error(("\rTest", CrLf))));
    }
}
