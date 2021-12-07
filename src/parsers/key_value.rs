use nom::branch::alt;
use nom::bytes::complete::{is_a, tag};
use nom::character::complete::line_ending;
use nom::combinator::{eof, map, recognize};
use nom::error::ParseError;
use nom::multi::separated_list1;
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;

use crate::parsers::string::{basic_string, literal_string};
use crate::parsers::whitespace::whitespace;
use crate::parsers::{toml_value, TomlValue};

#[derive(Debug, PartialEq)]
struct KeyValue(String, TomlValue);

fn bare_key<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    is_a("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-")(input)
}

fn dotted_key<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(separated_list1(tag("."), alt((bare_key, quoted_key))))(input)
}

fn quoted_key<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    alt((literal_string, basic_string))(input)
}

pub(crate) fn key<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    alt((dotted_key, quoted_key, bare_key))(input)
}

fn key_val_pair<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, KeyValue, E> {
    map(
        terminated(
            preceded(
                whitespace,
                separated_pair(key, tuple((whitespace, tag("="), whitespace)), toml_value),
            ),
            pair(whitespace, alt((eof, line_ending))),
        ),
        |(k, v)| KeyValue(k.to_string(), v),
    )(input)
}

#[cfg(test)]
mod tests_key_value {
    use nom::error::ErrorKind;

    use super::*;

    #[test]
    fn test_key_val_pair() {
        // ToDo: escaped strings are buggy
        assert_eq!(
            key_val_pair::<(&str, ErrorKind)>("key = true"),
            Ok(("", KeyValue("key".to_string(), TomlValue::Boolean(true))))
        );
        assert_eq!(
            key_val_pair::<(&str, ErrorKind)>("key = false"),
            Ok(("", KeyValue("key".to_string(), TomlValue::Boolean(false))))
        );
        assert_eq!(
            key_val_pair::<(&str, ErrorKind)>("key = 12"),
            Ok(("", KeyValue("key".to_string(), TomlValue::Integer(12))))
        );
        assert_eq!(
            key_val_pair::<(&str, ErrorKind)>("key = 12.2"),
            Ok(("", KeyValue("key".to_string(), TomlValue::Float(12.2))))
        );
    }
}
