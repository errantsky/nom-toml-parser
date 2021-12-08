use nom::bytes::complete::take_while;
use nom::character::complete::space0;
use nom::error::ParseError;
use nom::IResult;

pub(crate) fn whitespace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    space0(input)
}

// Taken from https://github.com/Geal/nom/blob/5405e1173f1052f7e006dcb0b9cfda2b06557b65/examples/json.rs
pub(crate) fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    // nom combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(move |c| chars.contains(c))(i)
}

#[cfg(test)]
mod tests_whitespace {
    use nom::character::complete::line_ending;
    use nom::Err;
    use nom::error::ErrorKind;
    use nom::error::ErrorKind::CrLf;

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
