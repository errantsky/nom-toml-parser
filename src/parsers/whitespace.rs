use nom::character::complete::space0;
use nom::error::ParseError;
use nom::IResult;

pub(crate) fn whitespace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    space0(input)
}

#[cfg(test)]
mod tests_whitespace {
    use nom::character::complete::line_ending;
    use nom::error::ErrorKind;
    use nom::error::ErrorKind::CrLf;
    use nom::Err;

    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(
            whitespace::<(&str, ErrorKind)>(" \tSomeText"),
            Ok(("SomeText", " \t"))
        );
        assert_eq!(whitespace::<(&str, ErrorKind)>("Test"), Ok(("Test", "")));
    }

    // #[test]
    // fn test_newline() {
    //     assert_eq!(line_ending::<&str, ErrorKind>("\nTest"), Ok(("Test", "\n")));
    //     assert_eq!(line_ending::<&str, ErrorKind>("\r\nTest"), Ok(("Test", "\r\n")));
    //     assert_eq!(line_ending::<&str, ErrorKind>("\rTest"), Err(Err::Error(("\rTest", CrLf))));
    // }
}
