use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::many0;
use nom::sequence::delimited;

use array::{array, Array};
use boolean::boolean;
use datetime::datetime;
use float::float;
use integer::integer;
use key_value::key_val_pair;
use nom_string::parse_string;
use table::table;

use crate::parsers::key_value::KeyValue;
use crate::parsers::table::Table;
use crate::parsers::whitespace::sp;

mod array;
mod boolean;
mod comment;
mod datetime;
mod float;
mod integer;
mod key_value;
mod nom_string;
mod string;
mod table;
mod whitespace;

// ToDo: find out how test files should be organized
// ToDo: should common imports be declared at the mod.rs file?
// ToDo: add documentation

fn toml_value<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(
    input: &'a str,
) -> IResult<&'a str, TomlValue, E> {
    alt((
        float,
        integer,
        boolean,
        map(parse_string, |s| TomlValue::Str(s)),
        // datetime,
        array,
    ))(input)
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
    Array(Box<Array>),
    InlineTable,
}

// enum TomlType {
//     Table(Table),
//     KeyValPair(KeyValue),
// }

fn cargo_root<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(
    input: &'a str,
) -> IResult<&'a str, Vec<Table>, E> {
    delimited(sp, many0(table), opt(sp))(input)
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use nom::error::ErrorKind;

    use super::*;

    #[test]
    fn test_cargo_self() {
        let input = read_to_string("assets/cargo_examples/self.toml").unwrap();
        println!("{:?}", cargo_root::<(&str, ErrorKind)>(&input));
    }

    // ToDo: Multiline array
    #[test]
    fn test_cargo_lalrpop() {
        let input = read_to_string("assets/cargo_examples/lalrpop.toml").unwrap();
        println!("{:?}", cargo_root::<(&str, ErrorKind)>(&input));
    }

    #[test]
    // ToDo: inline tables
    fn test_cargo_expand() {
        let input = read_to_string("assets/cargo_examples/cargo-expand.toml").unwrap();
        println!("{:?}", cargo_root::<(&str, ErrorKind)>(&input));
    }

    #[test]
    // ToDo: inline comment
    fn test_cargo_nom_locate() {
        let input = read_to_string("assets/cargo_examples/nom-locate.toml").unwrap();
        println!("{:?}", cargo_root::<(&str, ErrorKind)>(&input));
    }

    #[test]
    fn test_cargo_nom_supreme() {
        let input = read_to_string("assets/cargo_examples/nom-supreme.toml").unwrap();
        println!("{:?}", cargo_root::<(&str, ErrorKind)>(&input));
    }

    #[test]
    fn test_cargo_pyo3() {
        let input = read_to_string("assets/cargo_examples/pyo3.toml").unwrap();
        println!("{:?}", cargo_root::<(&str, ErrorKind)>(&input));
    }
}
