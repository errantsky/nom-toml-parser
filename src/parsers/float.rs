use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::one_of;
use nom::combinator::{map, opt};
use nom::error::ParseError;
use nom::IResult;
use nom::sequence::{pair, tuple};

use crate::parsers::integer::{decimal_integer, underscored_decimal};
use crate::parsers::TomlValue;

fn underscored_float<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, String, E> {
    map(
        tuple((underscored_decimal, tag("."), underscored_decimal)),
        |(left, dot, right)| format!("{}{}{}", left, dot, right),
    )(input)
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
pub(crate) fn float<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
    // ToDo run more tests to verify parser ordering
    alt((
        map(tag("+inf"), |_| TomlValue::Float(f64::INFINITY)),
        map(tag("inf"), |_| TomlValue::Float(f64::INFINITY)),
        map(tag("-inf"), |_| TomlValue::Float(f64::NEG_INFINITY)),
        map(tag("+nan"), |_| TomlValue::Float(f64::NAN)),
        map(tag("nan"), |_| TomlValue::Float(f64::NAN)),
        map(tag("-nan"), |_| TomlValue::Float(f64::NAN)),
        map(fractional_float, TomlValue::Float),
        map(exponential_float, TomlValue::Float),
        map(expo_frac_float, TomlValue::Float),
    ))(input)
}

#[cfg(test)]
mod tests_float {
    use nom::error::ErrorKind;

    use super::*;

    // ToDo: Add invalid floats
    // # INVALID FLOATS
    // invalid_float_1 = .7
    // invalid_float_2 = 7.
    // invalid_float_3 = 3.e+20
    // should throw and error
    // assert_eq!(
    //     underscored_float::<(&str, ErrorKind)>(".1123"),
    //     Ok(("", ".1123".to_string()))
    // );
    #[test]
    fn test_underscored_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("224_617.445_991_228"),
            Ok(("", TomlValue::Float(224617.445991228)))
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
    fn test_plus_signed_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("+1.0"),
            Ok(("", TomlValue::Float(1.0)))
        );
    }

    #[test]
    fn test_unsigned_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("3.1415"),
            Ok(("", TomlValue::Float(3.1415)))
        );
    }

    #[test]
    fn test_minus_signed_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("-0.01"),
            Ok(("", TomlValue::Float(-0.01)))
        );
    }

    #[test]
    fn test_plus_signed_zero_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("+0.0"),
            Ok(("", TomlValue::Float(0.0)))
        );
    }

    #[test]
    fn test_minus_signed_zero_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("-0.0"),
            Ok(("", TomlValue::Float(-0.0)))
        );
    }

    #[test]
    fn test_unsigned_nan_float() {
        match float::<(&str, ErrorKind)>("nan") {
            Ok(("", TomlValue::Float(num))) => assert!(f64::is_nan(num)),
            _ => panic!("nan testing went wrong."),
        }
    }

    #[test]
    fn test_plus_signed_nan_float() {
        match float::<(&str, ErrorKind)>("+nan") {
            Ok(("", TomlValue::Float(num))) => assert!(f64::is_nan(num)),
            _ => panic!("nan testing went wrong."),
        }
    }

    #[test]
    fn test_minus_signed_nan_float() {
        match float::<(&str, ErrorKind)>("-nan") {
            Ok(("", TomlValue::Float(num))) => assert!(f64::is_nan(num)),
            _ => panic!("nan testing went wrong."),
        }
    }

    #[test]
    fn test_unsigned_inf_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("inf"),
            Ok(("", TomlValue::Float(f64::INFINITY)))
        );
    }

    #[test]
    fn test_plus_signed_inf_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("+inf"),
            Ok(("", TomlValue::Float(f64::INFINITY)))
        );
    }

    #[test]
    fn test_minus_signed_inf_float() {
        assert_eq!(
            float::<(&str, ErrorKind)>("-inf"),
            Ok(("", TomlValue::Float(f64::NEG_INFINITY)))
        );
    }
}
