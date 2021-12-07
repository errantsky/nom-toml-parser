use nom::character::complete::{char, line_ending, not_line_ending};
use nom::error::ParseError;
use nom::sequence::{preceded, terminated};
use nom::IResult;

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
/// ToDo: Should the newline be consumed?
/// ToDo: What if the comment is in the last line of the file?
fn comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(char('#'), terminated(not_line_ending, line_ending))(input)
}

#[cfg(test)]
mod tests_comment {
    use nom::error::ErrorKind;

    use super::*;

    #[test]
    fn test_comment() {
        assert_eq!(
            comment::<(&str, ErrorKind)>("# This is a full-line comment\n"),
            Ok(("", " This is a full-line comment"))
        );
    }
}
