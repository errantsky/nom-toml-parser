use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::combinator::{eof, map};
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated};

use crate::parsers::key_value::{key, key_val_pair, KeyValue};

// ToDo: Fix string key val support

fn table_header<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    terminated(delimited(tag("["), key, tag("]")), alt((line_ending, eof)))(input)
}

/// Returns all of the key value pairs belonging to a table
/// Key value pairs can be separated and delimited by a variable number of
/// newlines (`\n` or `\r\n`). The last key pair can also have no newline
///  or eof as that can be taken by `table` parser.
fn table_body<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(input: &'a str) -> IResult<&'a str, Vec<KeyValue>, E> {
    terminated(
        many0(preceded(many0(line_ending), key_val_pair)),
        many0(line_ending),
    )(input)
}

#[derive(Debug, PartialEq)]
struct Table {
    header: String,
    key_val_vec: Vec<KeyValue>,
}

// ToDo: does terminate consume the termination slice?
fn table<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(input: &'a str) -> IResult<&'a str, Table, E> {
    map(
        terminated(pair(table_header, table_body), alt((eof, table_header))),
        |(header, key_val_vec)| {
            let header = header.to_string();
            Table {
                header,
                key_val_vec,
            }
        },
    )(input)
}

#[cfg(test)]
mod tests_table {
    use std::fs::read_to_string;

    use nom::character::complete::digit1;
    use nom::error::ErrorKind;

    use crate::parsers::TomlValue;
    use crate::parsers::TomlValue::{Boolean, Integer};

    use super::*;

    #[test]
    fn test_table_header() {
        assert_eq!(
            table_header::<(&str, ErrorKind)>("[table]\n"),
            Ok(("", "table"))
        )
    }

    #[test]
    fn test_table_body() {
        let input = read_to_string("assets/table-no-header.toml").unwrap();
        assert_eq!(
            table_body::<(&str, ErrorKind)>(&input),
            Ok((
                "",
                vec![
                    KeyValue("key1".to_string(), Boolean(false)),
                    KeyValue("key2".to_string(), Integer(123)),
                ]
            ))
        )
    }

    #[test]
    fn test_empty_table() {
        let input = read_to_string("assets/empty-table.toml").unwrap();
        assert_eq!(
            table::<(&str, ErrorKind)>(&input),
            Ok((
                "",
                Table {
                    header: "table-1".to_string(),
                    key_val_vec: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_single_table() {
        let input = read_to_string("assets/single-table.toml").unwrap();

        assert_eq!(
            table::<(&str, ErrorKind)>(&input),
            Ok((
                "",
                Table {
                    header: "table-1".to_string(),
                    key_val_vec: vec![KeyValue("key1".to_string(), TomlValue::Float(1.23))],
                }
            ))
        )
    }

    #[test]
    fn test_single_table_key_last_line() {
        let input = read_to_string("assets/table-key-last-line.toml").unwrap();

        assert_eq!(
            table::<(&str, ErrorKind)>(&input),
            Ok((
                "",
                Table {
                    header: "table".to_string(),
                    key_val_vec: vec![KeyValue("key".to_string(), TomlValue::Integer(123))],
                }
            ))
        )
    }
}
