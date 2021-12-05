use nom::branch::alt;
use nom::bytes::complete::{is_a, tag};
use nom::character::complete::{
    char, digit1, hex_digit1, line_ending, not_line_ending, oct_digit1, one_of, space0,
};
use nom::combinator::{map, opt};
use nom::error::ParseError;
use nom::IResult;
use nom::multi::separated_list1;
use nom::sequence::{pair, preceded, terminated};

#[derive(Debug, PartialEq)]
enum TomlValue {
    Str(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    OffsetDateTime,
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

fn float() {}

fn string() {}

// key value pair
fn key_val_pair() {}

#[cfg(test)]
mod tests {
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
}
