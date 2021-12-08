use std::fmt::{Display, Formatter};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, separated_pair, tuple};

use crate::parsers::{toml_value, TomlValue};
use crate::parsers::key_value::{key, KeyValue};
use crate::parsers::whitespace::{sp, whitespace};

#[derive(Debug, PartialEq)]
pub(crate) struct InlineTable {
    value: Option<KeyValue>,
    children: Option<Vec<InlineTable>>,
}

impl Display for InlineTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        match &self.value {
            Some(key_val) => output.push_str(&format!("\t{}: {}", key_val.0, key_val.1)),
            None => {}
        }
        match &self.children {
            Some(vec) => {
                output.push_str("\t[\n");
                for it in vec {
                    output.push_str(&format!("\t\t{}\n", &it.to_string()));
                }
                output.push_str("\t]\n");
            },
            None => {}
        }

        f.write_str(&output)
    }
}

pub(crate) fn inline_key_val_pair<
    'a,
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
>(
    input: &'a str,
) -> IResult<&'a str, KeyValue, E> {
    map(
        separated_pair(key, tuple((whitespace, tag("="), whitespace)), toml_value),
        |(k, v)| KeyValue(k.to_string(), v),
    )(input)
}

/// Parses inline table items that are not inline tables themselves
fn inline_table_key_val_value<
    'a,
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
>(
    input: &'a str,
) -> IResult<&'a str, InlineTable, E> {
    map(inline_key_val_pair, |key_val| InlineTable {
        value: Some(key_val),
        children: None,
    })(input)
}

fn inline_table_value<
    'a,
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
>(
    input: &'a str,
) -> IResult<&'a str, InlineTable, E> {
    map(
        delimited(
            pair(tag("{"), sp),
            separated_list1(
                delimited(sp, tag(","), sp),
                /// Each array item is either another array or a single TOML value, so test for both
                alt((inline_table_value, inline_table_key_val_value)),
            ),
            pair(sp, tag("}")),
        ),
        |inline_values| InlineTable {
            value: None,
            children: Some(inline_values),
        },
    )(input)
}

pub(crate) fn inline_table<
    'a,
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
>(
    input: &'a str,
) -> IResult<&'a str, TomlValue, E> {
    map(inline_table_value, |t| TomlValue::InlineTable(Box::new(t)))(input)
}

#[cfg(test)]
mod tests_inline_table {
    use nom::error::ErrorKind;

    use crate::parsers::array::Array;

    use super::*;

    #[test]
    fn test_inline_table1() {
        let input = r#"{ version = "1.0", features = ["derive"] }"#;
        let expected = Ok((
            "",
            InlineTable {
                value: None,
                children: Some(vec![
                    InlineTable {
                        value: Some(KeyValue(
                            String::from("version"),
                            TomlValue::Str(String::from("1.0")),
                        )),
                        children: None,
                    },
                    InlineTable {
                        value: Some(KeyValue(
                            String::from("features"),
                            TomlValue::Array(Box::new(Array {
                                value: None,
                                children: Some(vec![Array {
                                    value: Some(TomlValue::Str(String::from("derive"))),
                                    children: None,
                                }]),
                            })),
                        )),
                        children: None,
                    },
                ]),
            },
        ));
        assert_eq!(inline_table_value::<(&str, ErrorKind)>(input), expected);
    }
}
