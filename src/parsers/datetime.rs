use chrono::{DateTime, ParseResult};
use nom::{Err, IResult};
use nom::error::{Error, ErrorKind, ParseError};

use crate::parsers::TomlValue;

// fn t_delimited_offset_datetime<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, TomlValue, E> {
// //     1979-05-27 07:32:00Z
//     separated_pair(is_a('-0123456789), " ", is_a(':))
// }

fn offset_datetime(input: &str) -> IResult<&str, TomlValue, Error<&str>> {
    match DateTime::parse_from_rfc3339(input) {
        ParseResult::Ok(dt) => IResult::Ok(("", TomlValue::OffsetDateTime(dt))),
        // https://stackoverflow.com/a/70240688/14311849
        ParseResult::Err(e) => Err(Err::Error(Error::from_error_kind(input, ErrorKind::Fail))),
    }
}

fn local_date() {}

fn local_time() {}

fn local_datetime() {}

#[cfg(test)]
mod tests_datetime {
    use nom::error::ErrorKind;

    use super::*;

    #[test]
    fn test_offset_datetime() {
        println!("{:?}", DateTime::parse_from_rfc3339("1979-05-27T07:32:00Z"));
        println!(
            "{:?}",
            offset_datetime("1979-05-27T07:32:00Z")
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
