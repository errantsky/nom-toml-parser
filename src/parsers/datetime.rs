use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, ParseResult};
use nom::{Err, IResult};
use nom::branch::alt;
use nom::combinator::map;
use nom::error::{Error, ErrorKind, ParseError};

use crate::parsers::TomlValue;

// fn t_delimited_offset_datetime<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
// //     1979-05-27 07:32:00Z
//     separated_pair(is_a('-0123456789), " ", is_a(':))
// }

fn offset_datetime<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, DateTime<FixedOffset>, Error<&'a str>> {
    match DateTime::parse_from_rfc3339(input) {
        ParseResult::Ok(dt) => IResult::Ok(("", dt)),
        // https://stackoverflow.com/a/70240688/14311849
        // ToDo: extract chrono's error text and use it
        ParseResult::Err(e) => Err(Err::Error(Error::from_error_kind(input, ErrorKind::Fail))),
    }
}

fn local_date<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, NaiveDate, Error<&'a str>> {
    match NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        ParseResult::Ok(dt) => IResult::Ok(("", dt)),
        ParseResult::Err(e) => Err(Err::Error(Error::from_error_kind(input, ErrorKind::Fail))),
    }
}

fn local_time<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, NaiveTime, Error<&'a str>> {
    match NaiveTime::parse_from_str(input, "%H:%M:%S%.f") {
        ParseResult::Ok(dt) => IResult::Ok(("", dt)),
        ParseResult::Err(e) => Err(Err::Error(Error::from_error_kind(input, ErrorKind::Fail))),
    }
}

fn local_datetime<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, NaiveDateTime, Error<&'a str>> {
    match NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S%.f") {
        ParseResult::Ok(dt) => IResult::Ok(("", dt)),
        ParseResult::Err(e) => Err(Err::Error(Error::from_error_kind(input, ErrorKind::Fail))),
    }
}

pub(crate) fn datetime<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, TomlValue, Error<&'a str>> {
    alt((
        map(offset_datetime::<'a, E>, |x| TomlValue::OffsetDateTime(x)),
        map(local_datetime::<'a, E>, |x| TomlValue::LocalDateTime(x)),
        map(local_date::<'a, E>, |x| TomlValue::LocalDate(x)),
        map(local_time::<'a, E>, |x| TomlValue::LocalTime(x)),
    ))(input)
}

#[cfg(test)]
mod tests_datetime {
    use nom::error::ErrorKind;

    use super::*;

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
        println!("{:?}", NaiveDate::parse_from_str("1979-05-27", "%Y-%m-%d"));
        println!("{:?}", NaiveTime::parse_from_str("07:32:00", "%H:%M:%S"));
        println!(
            "{:?}",
            NaiveTime::parse_from_str("00:32:00.999999", "%H:%M:%S%.f")
        );

        // ToDo: The example below should be permitted based on RFC 3339 section 5.6, but it is not
        // ToDo: Add support for space delimited rfc3339 datetimes
        println!("{:?}", DateTime::parse_from_rfc3339("1979-05-27 07:32:00Z"));
    }
}
