use nom::branch::alt;
use nom::bytes::complete::{is_a, tag};
use nom::character::complete::{char, digit1, hex_digit1, oct_digit1, one_of};
use nom::combinator::{map, opt};
use nom::error::ParseError;
use nom::IResult;
use nom::multi::separated_list1;
use nom::sequence::{pair, preceded};

use crate::parsers::TomlValue;

// ToDo: have digit parser function as an argument to have a single `underscored_` func
/// Matches a `_` separated sequence of digits and returns them without underscores
pub(crate) fn underscored_decimal<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, String, E> {
    map(separated_list1(tag("_"), digit1), |vec| vec.concat())(input)
}

fn underscored_hex<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
    map(separated_list1(tag("_"), hex_digit1), |vec| vec.concat())(input)
}

fn underscored_oct<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
    map(separated_list1(tag("_"), oct_digit1), |vec| vec.concat())(input)
}

fn underscored_binary<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
    map(separated_list1(tag("_"), is_a("01")), |vec| vec.concat())(input)
}

pub(crate) fn decimal_integer<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, i64, E> {
    map(
        pair(opt(one_of("-+")), underscored_decimal),
        |(sign, x)| match sign {
            Some('-') => -x.parse::<i64>().unwrap(),
            _ => x.parse::<i64>().unwrap(),
        },
    )(input)
}

/// ToDo: handle parse error propagation
fn hex_integer<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, i64, E> {
    map(
        preceded(opt(char('+')), preceded(tag("0x"), underscored_hex)),
        |x| i64::from_str_radix(x.as_str(), 16).unwrap(),
    )(input)
}

fn oct_integer<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, i64, E> {
    map(
        preceded(opt(char('+')), preceded(tag("0o"), underscored_oct)),
        |x| i64::from_str_radix(x.as_str(), 8).unwrap(),
    )(input)
}

fn binary_integer<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, i64, E> {
    map(
        preceded(opt(char('+')), preceded(tag("0b"), underscored_binary)),
        |x| i64::from_str_radix(x.as_str(), 2).unwrap(),
    )(input)
}

///
///
/// ToDo: Proper error for numbers that cannot be represented losslessly
pub(crate) fn integer<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, TomlValue, E> {
    alt((
        map(binary_integer, TomlValue::Integer),
        map(oct_integer, TomlValue::Integer),
        map(hex_integer, TomlValue::Integer),
        map(decimal_integer, TomlValue::Integer),
    ))(input)
}

#[cfg(test)]
mod tests_integer {
    use nom::error::ErrorKind;

    use super::*;

// ToDo: Add failing tests

    #[test]
    fn test_underscored_decimal_number() {
        // ToDo: Add failing cases with leading or terminating underscores
        assert_eq!(
            underscored_decimal::<(&str, ErrorKind)>("1_2_3_4"),
            Ok(("", "1234".to_string()))
        );
    }

    #[test]
    fn test_underscored_hex_number() {
        assert_eq!(
            underscored_hex::<(&str, ErrorKind)>("dead_beef"),
            Ok(("", "deadbeef".to_string()))
        );
    }

    #[test]
    fn test_underscored_zero_leading_decimal_number() {
        assert_eq!(
            underscored_oct::<(&str, ErrorKind)>("0_1234567"),
            Ok(("", "01234567".to_string()))
        );
    }

    #[test]
    fn test_underscored_binary_number() {
        assert_eq!(
            underscored_binary::<(&str, ErrorKind)>("10_10_00010"),
            Ok(("", "101000010".to_string()))
        );
    }

    #[test]
    fn test_signed_positive_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("+99"),
            Ok(("", TomlValue::Integer(99)))
        );
    }

    #[test]
    fn test_unsigned_positive_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("42"),
            Ok(("", TomlValue::Integer(42)))
        );
    }

    #[test]
    fn test_zero_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("0"),
            Ok(("", TomlValue::Integer(0)))
        );
    }

    #[test]
    fn test_plus_signed_zero_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("+0"),
            Ok(("", TomlValue::Integer(0)))
        );
    }

    #[test]
    fn test_minus_signed_zero_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("-0"),
            Ok(("", TomlValue::Integer(0)))
        );
    }

    #[test]
    fn test_negative_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("-7"),
            Ok(("", TomlValue::Integer(-7)))
        );
    }

    #[test]
    fn test_uppercase_hex_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("0xDEADBEEF"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
    }

    #[test]
    fn test_lowercase_hex_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("0xdeadbeef"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
    }

    #[test]
    fn test_zero_leading_octal_integer1() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("0o01234567"),
            Ok(("", TomlValue::Integer(0o01234567)))
        );
    }

    #[test]
    fn test_zero_leading_octal_integer2() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("0o0755"),
            Ok(("", TomlValue::Integer(0o755)))
        );
    }

    #[test]
    fn test_octal_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("0o755"),
            Ok(("", TomlValue::Integer(0o755)))
        );
    }

    #[test]
    fn test_binary_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("0b11010110"),
            Ok(("", TomlValue::Integer(0b11010110)))
        );
    }

    #[test]
    fn test_underscored_decimal_integer1() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("5_349_221"),
            Ok(("", TomlValue::Integer(5349221)))
        );
    }

    #[test]
    fn test_underscored_decimal_integer2() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("53_49_221"),
            Ok(("", TomlValue::Integer(5349221)))
        );
    }

    #[test]
    fn test_underscored_decimal_integer3() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("1_2_3_4_5"),
            Ok(("", TomlValue::Integer(12345)))
        );
    }

    #[test]
    fn test_underscored_hex_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("0xdead_beef"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
    }

    #[test]
    fn test_plus_signed_underscored_hex_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("+0xdead_beef"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
    }

    #[test]
    fn test_plus_signed_zero_leading_underscored_hex_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("+0x0dead_beef"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
    }

    #[test]
    fn test_plus_signed_octal_integer() {
        assert_eq!(
            integer::<(&str, ErrorKind)>("+0o755"),
            Ok(("", TomlValue::Integer(0o755)))
        );
    }
}
