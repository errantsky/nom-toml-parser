use nom::branch::alt;
use nom::bytes::complete::{tag, take_until1};
use nom::character::complete::{char, line_ending};
use nom::combinator::{map, opt};
use nom::error::ParseError;
use nom::sequence::{pair, preceded};
use nom::IResult;

use crate::parsers::TomlValue;

// ToDo: Check for allowed control sequences
pub(crate) fn basic_string<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    preceded(tag("\""), take_until1("\""))(input)
}

// ToDo: line ending backslash
pub(crate) fn multiline_basic_string<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    preceded(pair(tag("\"\"\""), opt(line_ending)), take_until1("\"\"\""))(input)
}

pub(crate) fn literal_string<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    preceded(char('\''), take_until1("'"))(input)
}

pub(crate) fn multiline_literal_string<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    preceded(pair(tag("'''"), opt(line_ending)), take_until1("'''"))(input)
}

pub(crate) fn string<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
    map(
        alt((
            basic_string,
            multiline_basic_string,
            literal_string,
            multiline_literal_string,
        )),
        |s| TomlValue::Str(s.to_string()),
    )(input)
}
