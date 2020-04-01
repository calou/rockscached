use nom;

use std::str::FromStr;
use crate::command::Command;
use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while, take_while1, is_not},
    number::streaming::be_u64,
    sequence::tuple,
    branch::alt,
    character::complete::{crlf, space1, digit1},
};

#[derive(PartialEq, Debug)]
struct RawCommand<'a> {
    pub verb: String,
    pub args: Vec<&'a[u8]>,
}

fn not_space(s: &[u8]) -> IResult<&[u8], &[u8]> {
    is_not(" \t\r\n")(s)
}

fn _parse_set<'a>(input: &'a [u8]) -> IResult<&'a [u8], (&[u8], &[u8], &[u8], &[u8], &[u8])> {
    let (input, (v, _, k, _, f, _,   e, _, val, _)) = tuple((tag("set"), space1, not_space, space1, not_space, space1, digit1, space1, take_until("\r\n"), crlf))(input)?;
    Ok((input, (v, k, f, e, val)))
}

fn parse_set<'a>(input: &'a [u8]) -> IResult<&'a[u8], RawCommand<'_>> {
    match _parse_set(input){
        Ok((input, (v, key, flags, expiration_timestamp, value))) => {
                Ok((input, RawCommand {verb: String::from_utf8(v.to_vec()).unwrap(), args: vec![key, flags, expiration_timestamp, value]}))
            },
        Err(e) => Result::Err(e)
    }
}

fn _parse_get<'a>(input: &'a [u8]) -> IResult<&'a [u8], (&[u8], &[u8])> {
    let (input, (v, _, k, _)) = tuple((tag("get"), space1, not_space, crlf))(input)?;
    Ok((input, (v,k)))
}

fn parse_get<'a>(input: &'a [u8]) -> IResult<&'a[u8], RawCommand<'_>> {
    match _parse_get(input){
        Ok((input, (v, key))) => {
            Ok((input, RawCommand {verb: String::from_utf8(v.to_vec()).unwrap(), args: vec![key]}))
        },
        Err(e) => Result::Err(e)
    }
}

fn parse_raw_command<'a>(input: &'a [u8]) -> IResult<&'a[u8], RawCommand<'_>> {
    let (input, cmd) = alt((parse_get, parse_set))(input)?;
    Ok((input, cmd))
}

pub fn parse(input: &[u8]) -> Result<Command, String> {
    match parse_raw_command(input){
        Ok((_input, cmd)) => {
            match cmd.verb.as_str() {
                "get" => Ok(Command::MGet {key: cmd.args[0]}),
                "set" => {
                    // TODO remove this ugly code
                    let x = String::from_utf8(cmd.args[2].to_vec()).unwrap();
                    let ttl = u64::from_str(x.as_str()).unwrap();
                    Ok(Command::MSet {key: cmd.args[0], flags: cmd.args[1], ttl, value: cmd.args[3]})
                },
                _ => Err(String::from("Invalid command"))
            }

        }
        _ => Err(String::from("Unable to parse command"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_set_simple() {
        let (_input, parts) = _parse_set(b"set myKey 0 60 the value to store\r\n").ok().unwrap();
        assert_eq!(parts.0, b"set");
        assert_eq!(parts.1, b"myKey");
        assert_eq!(parts.2, b"0");
        assert_eq!(parts.3, b"60");
        assert_eq!(parts.4, b"the value to store");
    }

    #[test]
    fn parse_set_obj_simple() {
        let (_input, cmd) = parse_set(b"set myKey 0 60 the value to store\r\n").ok().unwrap();
        assert_eq!(cmd, RawCommand { verb: String::from("set"),  args: vec![b"myKey", b"0", b"60", b"the value to store"]});
    }

    #[test]
    fn parse_get_simple() {
        let (_input, (verb, key)) = _parse_get(b"get myKey\r\n").ok().unwrap();
        assert_eq!(verb, b"get");
        assert_eq!(key, b"myKey");
    }

    #[test]
    fn parse_get_obj_simple() {
        let (_input, cmd) = parse_get(b"get myKey\r\n").ok().unwrap();
        assert_eq!(cmd, RawCommand { verb: String::from("get"),  args: vec![b"myKey"]});
    }

    #[test]
    fn parse_raw_command_simple_get() {
        let (_input, cmd) = parse_raw_command(b"get myKey\r\n").ok().unwrap();
        assert_eq!(cmd, RawCommand { verb: String::from("get"),  args: vec![b"myKey"]});
    }

    #[test]
    fn parse_raw_command_simple_set() {
        let (_input, cmd) = parse_raw_command(b"set myKey 0 60 the value to store\r\n").ok().unwrap();
        assert_eq!(cmd, RawCommand { verb: String::from("set"),  args: vec![b"myKey", b"0", b"60", b"the value to store"]});
    }

    #[test]
    fn parse_invalid() {
        assert!(parse(b"INVALID").is_err());
    }

    #[test]
    fn parse_for_get() {
        let result = parse(b"get myKey\r\n");
        assert_eq!(result.unwrap(), Command::MGet {key: b"myKey"});
    }

    #[test]
    fn parse_for_set() {
        let result = parse(b"set myKey 0 60 the value to store\r\n");
        assert_eq!(result.unwrap(), Command::MSet {key: b"myKey", flags: b"0", ttl: 60u64, value: b"the value to store"});
    }
}
