use nom::{Err, IResult};
use nom::character::complete::{char, line_ending, not_line_ending, space0};
use nom::error::{ErrorKind, ErrorKind::CrLf, ParseError};
use nom::sequence::{preceded, terminated};

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

fn whitespace<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    space0(input)
}

fn newline<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    line_ending(input)
}

/// A hash symbol marks the rest of the line as a comment, except when inside a string.
/// ```Rust
/// # This is a full-line comment
/// key = "value"  # This is a comment at the end of a line
/// another = "# This is not a comment"
/// ```
///
/// Control characters other than tab (U+0000 to U+0008, U+000A to U+001F, U+007F) are not permitted
/// in comments.
/// ToDo: Test inline comments
/// ToDo: Test for control characters
fn comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(char('#'), terminated(not_line_ending, line_ending))(input)
}

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

    #[test]
    fn test_comment() {
        assert_eq!(comment::<(&str, ErrorKind)>("# This is a full-line comment\n"), Ok(("", " This is a full-line comment")));
    }
}
