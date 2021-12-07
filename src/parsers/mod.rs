use chrono::{DateTime, FixedOffset};
use nom::branch::alt;
use nom::error::ParseError;
use nom::IResult;

use boolean::boolean;
use float::float;
use integer::integer;
use string::string;

mod array;
mod boolean;
mod comment;
mod datetime;
mod float;
mod integer;
mod key_value;
mod string;
mod whitespace;

// ToDo: divide parsers into separate files
// ToDo: find out how test files should be organized
// ToDo: should common imports be declared at the mod.rs file?

fn toml_value<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
    alt((float, integer, boolean, string))(input)
}

#[derive(Debug, PartialEq)]
pub(crate) enum TomlValue {
    Str(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    OffsetDateTime(DateTime<FixedOffset>),
    LocalDateTime,
    LocalDate,
    LocalTime,
    Array,
    InlineTable,
}
