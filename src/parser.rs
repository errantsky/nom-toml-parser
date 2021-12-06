use chrono::{Date, DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, ParseResult};
use nom::{Err, IResult};
use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, take_while1};
use nom::character::complete::{
    char, digit1, hex_digit1, line_ending, not_line_ending, oct_digit1, one_of, space0,
};
use nom::combinator::{map, map_res, opt};
use nom::Err::Failure;
use nom::error::{context, Error, ErrorKind, ParseError};
use nom::multi::{many1, separated_list1};
use nom::number::complete::{double, f64};
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};

#[derive(Debug, PartialEq)]
enum TomlValue {
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

fn whitespace<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    space0(input)
}

fn newline<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    line_ending(input)
}

/// A hash symbol marks the rest of the line as a comment, except when inside a string.
/// ```Rust
/// # This is a full-line comment
/// key = "value"  # This is a comment at the end of a line
/// another = "# This is not a comment"
/// ```
///
/// Control characters other than tab (U+0000 to U+0008, U+000A to U+001F, U+007F) are not permitted
/// in comments.
/// ToDo: Test inline comments
/// ToDo: Test for control characters
/// ToDo: Should the newline be consumed?
/// ToDo: What if the comment is in the last line of the file?
fn comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    preceded(char('#'), terminated(not_line_ending, line_ending))(input)
}

// ToDo: have digit parser function as an argument to have a single `underscored_` func
/// Matches a `_` separated sequence of digits and returns them without underscores
fn underscored_decimal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
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

fn decimal_integer<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, i64, E> {
    map(
        pair(opt(one_of("-+")), underscored_decimal),
        |(sign, x)| match sign {
            Some('-') => -1 * x.parse::<i64>().unwrap(),
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
fn integer<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
    alt((
        map(binary_integer, TomlValue::Integer),
        map(oct_integer, TomlValue::Integer),
        map(hex_integer, TomlValue::Integer),
        map(decimal_integer, TomlValue::Integer),
    ))(input)
}

fn underscored_float<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
    map(separated_list1(tag("_"), is_a(".0123456789")), |vec| {
        vec.into_iter().collect()
    })(input)
}

fn exponential_float<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, f64, E> {
    map(
        tuple((
            decimal_integer,
            one_of("Ee"),
            opt(one_of("-+")),
            decimal_integer,
        )),
        |(left, e, sign, right)| match sign {
            Some('-') => left as f64 / 10_f64.powi(right as i32),
            _ => left as f64 * 10_f64.powi(right as i32),
        },
    )(input)
}

fn fractional_float<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, f64, E> {
    map(
        pair(opt(one_of("-+")), underscored_float),
        |(sign, x)| match sign {
            Some('-') => -1. * x.parse::<f64>().unwrap(),
            _ => x.parse::<f64>().unwrap(),
        },
    )(input)
}

fn expo_frac_float<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, f64, E> {
    map(
        tuple((
            fractional_float,
            one_of("Ee"),
            opt(one_of("-+")),
            decimal_integer,
        )),
        |(left, e, sign, right)| match sign {
            Some('-') => left / 10_f64.powi(right as i32),
            _ => left * 10_f64.powi(right as i32),
        },
    )(input)
}

/// Negative NaNs do not seem to currently exist in Rust
/// https://github.com/rust-lang/rust/issues/81261
/// For now, all NaNs map to `f64::NAN`
fn float<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
    // ToDo run more tests to verify parser ordering
    alt((
        map(tag("+inf"), |_| TomlValue::Float(f64::INFINITY)),
        map(tag("-inf"), |_| TomlValue::Float(f64::NEG_INFINITY)),
        map(tag("+nan"), |_| TomlValue::Float(f64::NAN)),
        map(tag("-nan"), |_| TomlValue::Float(f64::NAN)),
        map(fractional_float, TomlValue::Float),
        map(exponential_float, TomlValue::Float),
        map(expo_frac_float, TomlValue::Float),
        map(double, TomlValue::Float),
    ))(input)
}

fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
    alt((
        map(tag("true"), |_| TomlValue::Boolean(true)),
        map(tag("false"), |_| TomlValue::Boolean(false)),
    ))(input)
}

// fn t_delimited_offset_datetime<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
// //     1979-05-27 07:32:00Z
//     separated_pair(is_a('-0123456789), " ", is_a(':))
// }

fn offset_datetime<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
    match DateTime::parse_from_rfc3339(input) {
        ParseResult::Ok(dt) => IResult::Ok(("", TomlValue::OffsetDateTime(dt))),
        ParseResult::Err(e) => IResult::Ok(("", TomlValue::Integer(12))),
    }
}

fn local_date() {}

fn local_time() {}

fn local_datetime() {}

fn string() {}

// key value pair
fn key_val_pair() {}

#[cfg(test)]
mod tests {
    use chrono::Date;
    use nom::Err;
    use nom::error::ErrorKind;
    use nom::error::ErrorKind::CrLf;

    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(
            whitespace::<(&str, ErrorKind)>(" \tSomeText"),
            Ok(("SomeText", " \t"))
        );
        assert_eq!(whitespace::<(&str, ErrorKind)>("Test"), Ok(("Test", "")));
    }

    #[test]
    fn test_newline() {
        assert_eq!(newline::<(&str, ErrorKind)>("\nTest"), Ok(("Test", "\n")));
        assert_eq!(
            newline::<(&str, ErrorKind)>("\r\nTest"),
            Ok(("Test", "\r\n"))
        );
        assert_eq!(
            newline::<(&str, ErrorKind)>("\rTest"),
            Err(Err::Error(("\rTest", CrLf)))
        );
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            comment::<(&str, ErrorKind)>("# This is a full-line comment\n"),
            Ok(("", " This is a full-line comment"))
        );
    }

    #[test]
    fn test_underscored_number() {
        // ToDo: Add failing cases with leading or terminating underscores
        assert_eq!(
            underscored_decimal::<(&str, ErrorKind)>("1_2_3_4"),
            Ok(("", "1234".to_string()))
        );
        assert_eq!(
            underscored_hex::<(&str, ErrorKind)>("dead_beef"),
            Ok(("", "deadbeef".to_string()))
        );
        assert_eq!(
            underscored_oct::<(&str, ErrorKind)>("0_1234567"),
            Ok(("", "01234567".to_string()))
        );
        assert_eq!(
            underscored_binary::<(&str, ErrorKind)>("10_10_00010"),
            Ok(("", "101000010".to_string()))
        );
    }

    #[test]
    fn test_integer() {
        // ToDo: Add failing tests
        assert_eq!(
            integer::<(&str, ErrorKind)>("+99"),
            Ok(("", TomlValue::Integer(99)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("42"),
            Ok(("", TomlValue::Integer(42)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("0"),
            Ok(("", TomlValue::Integer(0)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("+0"),
            Ok(("", TomlValue::Integer(0)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("-0"),
            Ok(("", TomlValue::Integer(0)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("-7"),
            Ok(("", TomlValue::Integer(-7)))
        );
        // hexadecimal numbers
        assert_eq!(
            integer::<(&str, ErrorKind)>("0xDEADBEEF"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("0xdeadbeef"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
        // octal numbers
        assert_eq!(
            integer::<(&str, ErrorKind)>("0o01234567"),
            Ok(("", TomlValue::Integer(0o01234567)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("0o755"),
            Ok(("", TomlValue::Integer(0o755)))
        );
        // binary numbers
        assert_eq!(
            integer::<(&str, ErrorKind)>("0b11010110"),
            Ok(("", TomlValue::Integer(0b11010110)))
        );
        // readable underscores
        assert_eq!(
            integer::<(&str, ErrorKind)>("1_000"),
            Ok(("", TomlValue::Integer(1000)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("5_349_221"),
            Ok(("", TomlValue::Integer(5349221)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("53_49_221"),
            Ok(("", TomlValue::Integer(5349221)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("1_2_3_4_5"),
            Ok(("", TomlValue::Integer(12345)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("0xdead_beef"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
        // prefixed with plus sign
        assert_eq!(
            integer::<(&str, ErrorKind)>("+0xdead_beef"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("+0o755"),
            Ok(("", TomlValue::Integer(0o755)))
        );
        // prefixed with leading zero
        assert_eq!(
            integer::<(&str, ErrorKind)>("0o0755"),
            Ok(("", TomlValue::Integer(0o755)))
        );
        assert_eq!(
            integer::<(&str, ErrorKind)>("+0x0dead_beef"),
            Ok(("", TomlValue::Integer(0xDEADBEEF)))
        );
    }

    fn test_boolean() {
        assert_eq!(
            boolean::<(&str, ErrorKind)>("true"),
            Ok(("", TomlValue::Boolean(true)))
        );
        assert_eq!(
            boolean::<(&str, ErrorKind)>("false"),
            Ok(("", TomlValue::Boolean(false)))
        );
    }

    #[test]
    fn test_underscored_float() {
        assert_eq!(
            underscored_float::<(&str, ErrorKind)>("1123.0"),
            Ok(("", "1123.0".to_string()))
        );
        assert_eq!(
            underscored_float::<(&str, ErrorKind)>(".1123"),
            Ok(("", ".1123".to_string()))
        );
        assert_eq!(
            underscored_float::<(&str, ErrorKind)>("11_23.0"),
            Ok(("", "1123.0".to_string()))
        );
    }

    #[test]
    fn test_exponential_float() {
        assert_eq!(
            exponential_float::<(&str, ErrorKind)>("5e+22"),
            Ok(("", 5e+22))
        );
        assert_eq!(
            exponential_float::<(&str, ErrorKind)>("1e06"),
            Ok(("", 1e06))
        );
        assert_eq!(
            exponential_float::<(&str, ErrorKind)>("-2E-2"),
            Ok(("", -2E-2))
        );
    }

    #[test]
    fn test_expo_frac_float() {
        assert_eq!(
            expo_frac_float::<(&str, ErrorKind)>("6.626e-34"),
            Ok(("", 6.626e-34))
        );
    }

    #[test]
    fn test_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("+1.0"),
            Ok(("", TomlValue::Float(1.0)))
        );
        assert_eq!(
            float::<(&str, ErrorKind)>("3.1415"),
            Ok(("", TomlValue::Float(3.1415)))
        );
        assert_eq!(
            float::<(&str, ErrorKind)>("-0.01"),
            Ok(("", TomlValue::Float(-0.01)))
        );

        assert_eq!(
            float::<(&str, ErrorKind)>("+0.0"),
            Ok(("", TomlValue::Float(0.0)))
        );
        assert_eq!(
            float::<(&str, ErrorKind)>("-0.0"),
            Ok(("", TomlValue::Float(-0.0)))
        );

        match float::<(&str, ErrorKind)>("nan") {
            Ok(("", TomlValue::Float(num))) => assert!(f64::is_nan(num)),
            _ => panic!("nan testing went wrong."),
        }

        match float::<(&str, ErrorKind)>("+nan") {
            Ok(("", TomlValue::Float(num))) => assert!(f64::is_nan(num)),
            _ => panic!("nan testing went wrong."),
        }

        match float::<(&str, ErrorKind)>("-nan") {
            Ok(("", TomlValue::Float(num))) => assert!(f64::is_nan(num)),
            _ => panic!("nan testing went wrong."),
        }

        assert_eq!(
            float::<(&str, ErrorKind)>("inf"),
            Ok(("", TomlValue::Float(f64::INFINITY)))
        );
        assert_eq!(
            float::<(&str, ErrorKind)>("+inf"),
            Ok(("", TomlValue::Float(f64::INFINITY)))
        );
        assert_eq!(
            float::<(&str, ErrorKind)>("-inf"),
            Ok(("", TomlValue::Float(f64::NEG_INFINITY)))
        );
        // underscored floats
        assert_eq!(
            float::<(&str, ErrorKind)>("224_617.445_991_228"),
            Ok(("", TomlValue::Float(224617.445991228)))
        );

        // ToDo: Add invalid floats
        // # INVALID FLOATS
        // invalid_float_1 = .7
        // invalid_float_2 = 7.
        // invalid_float_3 = 3.e+20
    }

    #[test]
    fn test_offset_datetime() {
        println!("{:?}", DateTime::parse_from_rfc3339("1979-05-27T07:32:00Z"));
        println!(
            "{:?}",
            offset_datetime::<(&str, ErrorKind)>("1979-05-27T07:32:00Z")
        );
        println!(
            "{:?}",
            DateTime::parse_from_rfc3339("1979-05-27T00:32:00-07:00")
        );
        println!(
            "{:?}",
            DateTime::parse_from_rfc3339("1979-05-27T00:32:00.999999-07:00")
        );
        // The example below should be permitted based on RFC 3339 section 5.6, but it is not
        println!("{:?}", DateTime::parse_from_rfc3339("1979-05-27 07:32:00Z"));
    }
}
