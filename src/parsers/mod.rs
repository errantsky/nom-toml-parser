use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use nom::branch::alt;
use nom::combinator::map;
use nom::error::{FromExternalError, ParseError};
use nom::IResult;

use boolean::boolean;
use float::float;
use integer::integer;
use nom_string::parse_string;

mod array;
mod boolean;
mod comment;
mod datetime;
mod float;
mod integer;
mod key_value;
mod string;
mod table;
mod whitespace;
mod nom_string;

// ToDo: find out how test files should be organized
// ToDo: should common imports be declared at the mod.rs file?
// ToDo: add documentation

fn toml_value<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
    alt((float, integer, boolean, map(parse_string, |s| TomlValue::Str(s))))(input)
}

#[derive(Debug, PartialEq)]
pub(crate) enum TomlValue {
    Str(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    OffsetDateTime(DateTime<FixedOffset>),
    LocalDateTime(NaiveDateTime),
    LocalDate(NaiveDate),
    LocalTime(NaiveTime),
    // Array,
    // InlineTable,
}
