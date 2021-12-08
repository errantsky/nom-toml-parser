use std::fmt::{Display, Formatter};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, value};
use nom::error::{FromExternalError, ParseError};
use nom::IResult;
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, separated_pair, tuple};

use crate::parsers::{toml_value, TomlValue};
use crate::parsers::key_value::key;
use crate::parsers::whitespace::whitespace;

// ToDo: Should key be a concrete type?
// ToDo: Should arrays be a subset of key value pairs?
// ToDo: Pretty printing
// ToDo: Add multiline array definition support
/// Stores any data that a TOML array should can store, including other arrays
/// So, each item in an array is either a single value, such as a integer, or another array.
/// `Array` stores both types. For single values, the `value` optional field holds a
/// `TomlValue` and sets the `children` to `None`, while an array item sets `value` to `None`
///  and stores array data in the `children` optional field.
#[derive(Debug, PartialEq)]
pub(crate) struct Array {
    pub(crate) value: Option<TomlValue>,
    pub(crate) children: Option<Vec<Array>>,
}

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push_str(&format!("Array:\n"));
        match &self.value {
            Some(tv) => output.push_str(&format!("\t{}\n", &tv.to_string())),
            None => {}
        }
        match &self.children {
            Some(vec) => {
                output.push_str("\t[\n");
                for a in vec {
                    output.push_str(&format!("\t\t{}\n", &a.to_string()));
                }
                output.push_str("\t]\n");
            },
            None => {}
        }
        f.write_str(&output)
    }
}

/// Parses array items that are not arrays themselves
fn array_toml_value<
    'a,
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
>(
    input: &'a str,
) -> IResult<&'a str, Array, E> {
    map(toml_value, |toml_val| Array {
        value: Some(toml_val),
        children: None,
    })(input)
}

// ToDo: A general way to handle whitespace
// ToDo: Test single entry or empty arrays, with extraneous commas
/// A recursive parser to parses the right side of a TOML array definition such as "name = [1,2,3]"
fn array_value<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(
    input: &'a str,
) -> IResult<&'a str, Array, E> {
    map(
        delimited(
            pair(tag("["), whitespace),
            separated_list1(
                delimited(whitespace, tag(","), whitespace),
                /// Each array item is either another array or a single TOML value, so test for both
                alt((array_value, array_toml_value)),
            ),
            pair(whitespace, tag("]")),
        ),
        |array_values| Array {
            value: None,
            children: Some(array_values),
        },
    )(input)
}

/// Parses a TOML array definition such as "name = [1,2,3]"
pub(crate) fn array<
    'a,
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
>(
    input: &'a str,
) -> IResult<&'a str, TomlValue, E> {
    map(array_value, |v| TomlValue::Array(Box::new(v)))(input)
}

#[cfg(test)]
mod tests_array {
    use nom::error::ErrorKind;

    use super::*;

    #[test]
    fn test_no_whitespace_array_value() {
        assert_eq!(
            array_value::<(&str, ErrorKind)>("[1,2,3]"),
            Ok((
                "",
                Array {
                    value: None,
                    children: Some(vec![
                        Array {
                            value: Some(TomlValue::Integer(1)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(2)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(3)),
                            children: None,
                        },
                    ],)
                }
            ))
        );
    }

    #[test]
    fn test_whitespaced_integer_array() {
        assert_eq!(
            array_value::<(&str, ErrorKind)>("[ 1, 2, 3 ]"),
            Ok((
                "",
                Array {
                    value: None,
                    children: Some(vec![
                        Array {
                            value: Some(TomlValue::Integer(1)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(2)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(3)),
                            children: None,
                        },
                    ],)
                }
            ))
        );
    }

    #[test]
    fn test_integer_array() {
        assert_eq!(
            array::<(&str, ErrorKind)>("[ 1, 2, 3 ]"),
            Ok((
                "",
                TomlValue::Array(Box::new(Array {
                    value: None,
                    children: Some(vec![
                        Array {
                            value: Some(TomlValue::Integer(1)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(2)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(3)),
                            children: None,
                        },
                    ]),
                }))
            ))
        );
    }

    #[test]
    fn test_float_array() {
        // Test with an extra comma at the end
        assert_eq!(
            array::<(&str, ErrorKind)>("[ 0.1, 0.2, 0.5]"),
            Ok((
                "",
                TomlValue::Array(Box::new(Array {
                    value: None,
                    children: Some(vec![
                        Array {
                            value: Some(TomlValue::Float(0.1)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Float(0.2)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Float(0.5)),
                            children: None,
                        },
                    ]),
                }))
            ))
        );
    }

    #[test]
    fn test_mixed_integer_float_array() {
        // Test with an extra comma at the end
        assert_eq!(
            array::<(&str, ErrorKind)>("[ 0.1, 0.2, 0.5, 1, 2, 5 ]"),
            Ok((
                "",
                TomlValue::Array(Box::new(Array {
                    value: None,
                    children: Some(vec![
                        Array {
                            value: Some(TomlValue::Float(0.1)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Float(0.2)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Float(0.5)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(1)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(2)),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Integer(5)),
                            children: None,
                        },
                    ]),
                }))
            ))
        );
    }

    #[test]
    fn test_nested_integer_array() {
        assert_eq!(
            array::<(&str, ErrorKind)>("[ [ 1, 2 ], [3, 4, 5] ]"),
            Ok((
                "",
                TomlValue::Array(Box::new(Array {
                    value: None,
                    children: Some(vec![
                        Array {
                            value: None,
                            children: Some(vec![
                                Array {
                                    value: Some(TomlValue::Integer(1)),
                                    children: None,
                                },
                                Array {
                                    value: Some(TomlValue::Integer(2)),
                                    children: None,
                                },
                            ]),
                        },
                        Array {
                            value: None,
                            children: Some(vec![
                                Array {
                                    value: Some(TomlValue::Integer(3)),
                                    children: None,
                                },
                                Array {
                                    value: Some(TomlValue::Integer(4)),
                                    children: None,
                                },
                                Array {
                                    value: Some(TomlValue::Integer(5)),
                                    children: None,
                                },
                            ]),
                        },
                    ]),
                }))
            ))
        );
    }

    #[test]
    fn test_no_whitespace_string_array() {
        assert_eq!(
            array_value::<(&str, ErrorKind)>(r#"["red","yellow","green"]"#),
            Ok((
                "",
                Array {
                    value: None,
                    children: Some(vec![
                        Array {
                            value: Some(TomlValue::Str("red".to_string())),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Str("yellow".to_string())),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Str("green".to_string())),
                            children: None,
                        },
                    ], ),
                }
            ))
        );
    }

    #[test]
    fn test_basic_string_array() {
        assert_eq!(
            array::<(&str, ErrorKind)>(r#"[ "red", "yellow", "green" ]"#),
            Ok((
                "",
                TomlValue::Array(Box::new(Array {
                    value: None,
                    children: Some(vec![
                        Array {
                            value: Some(TomlValue::Str("red".to_string())),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Str("yellow".to_string())),
                            children: None,
                        },
                        Array {
                            value: Some(TomlValue::Str("green".to_string())),
                            children: None,
                        },
                    ]),
                }, ))
            ))
        );
    }

    #[test]
    fn test_nested_mixed_array() {
        println!(
            "{:?}",
            array::<(&str, ErrorKind)>(r#"[ [ 1, 2 ], ["a", "b", "c"] ]"#)
        );
    }

    #[test]
    fn test_single_string_array() {
        println!("{:?}", array::<(&str, ErrorKind)>(r#"["derive"]"#));
    }

    // #[test]
    // fn test_different_string_types_array() {
    //     println!("{:?}", array::<(&str, ErrorKind)>(r#"string_array = [ "all", 'strings', """are the same""", '''type''' ]"#));
    // }
}
