use std::fmt::{Display, format, Formatter};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::combinator::{eof, map, opt, peek};
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};

use crate::parsers::{toml_value, TomlValue};
use crate::parsers::comment::comment;
use crate::parsers::key_value::{key, key_val_pair, KeyValue};
use crate::parsers::whitespace::{sp, whitespace};

// ToDo: Fix string key val support

fn table_header<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    terminated(delimited(tag("["), key, tag("]")), alt((line_ending, eof)))(input)
}

/// Returns all of the key value pairs belonging to a table
/// Key value pairs can be separated and delimited by a variable number of
/// newlines (`\n` or `\r\n`). The last key pair can also have no newline
///  or eof as that can be taken by `table` parser.
fn table_body<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(
    input: &'a str,
) -> IResult<&'a str, Vec<KeyValue>, E> {
    terminated(
        many0(preceded(many0(alt((comment, line_ending))), key_val_pair)),
        many0(alt((comment, line_ending))),
    )(input)
}

#[derive(Debug, PartialEq)]
pub(crate) struct Table {
    header: String,
    key_val_vec: Vec<KeyValue>,
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push_str(&format!("Table: {}\n", &self.header));
        for key_val in &self.key_val_vec {
            output.push_str(&format!("\t{}: {}", key_val.0, key_val.1));
        }

        f.write_str(&output)
    }
}

// ToDo: does terminate consume the termination slice?
pub(crate) fn full_table<
    'a,
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
>(
    input: &'a str,
) -> IResult<&'a str, Table, E> {
    map(
        terminated(
            pair(table_header, table_body),
            peek(alt((eof, table_header))),
        ),
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

    use crate::parsers::array::Array;
    use crate::parsers::TomlValue;

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
                    KeyValue("key1".to_string(), TomlValue::Boolean(false)),
                    KeyValue("key2".to_string(), TomlValue::Integer(123)),
                ]
            ))
        )
    }

    #[test]
    fn test_empty_table() {
        let input = read_to_string("assets/empty-table.toml").unwrap();
        assert_eq!(
            full_table::<(&str, ErrorKind)>(&input),
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
            full_table::<(&str, ErrorKind)>(&input),
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
            full_table::<(&str, ErrorKind)>(&input),
            Ok((
                "",
                Table {
                    header: "table".to_string(),
                    key_val_vec: vec![KeyValue("key".to_string(), TomlValue::Integer(123))],
                }
            ))
        )
    }

    #[test]
    fn test_single_string_key_val_table() {
        let input = read_to_string("assets/sing-string-table.toml").unwrap();

        assert_eq!(
            full_table::<(&str, ErrorKind)>(&input),
            Ok((
                "",
                Table {
                    header: "table-1".to_string(),
                    key_val_vec: vec![KeyValue(
                        "key1".to_string(),
                        TomlValue::Str(String::from("this is a string")),
                    )],
                }
            ))
        )
    }
}
