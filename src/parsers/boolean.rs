use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::ParseError;
use nom::IResult;

use crate::parsers::TomlValue;

pub(crate) fn boolean<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, TomlValue, E> {
    alt((
        map(tag("true"), |_| TomlValue::Boolean(true)),
        map(tag("false"), |_| TomlValue::Boolean(false)),
    ))(input)
}

#[cfg(test)]
mod tests_boolean {
    use nom::error::ErrorKind;

    use super::*;

    #[test]
    fn test_true() {
        assert_eq!(
            boolean::<(&str, ErrorKind)>("true"),
            Ok(("", TomlValue::Boolean(true)))
        );
    }

    #[test]
    fn test_false() {
        assert_eq!(
            boolean::<(&str, ErrorKind)>("false"),
            Ok(("", TomlValue::Boolean(false)))
        );
    }
}
