use nom;

use std::str::FromStr;
use crate::command::Command;
use nom::{
    IResult,
    bytes::complete::{tag, take_until, is_not},
    sequence::tuple,
    branch::alt,
    character::complete::{crlf, space1, digit1},
};
use crate::byte_utils::bytes_to_u64;

#[derive(PartialEq, Debug)]
struct RawCommand<'a> {
    pub verb: String,
    pub args: Vec<&'a[u8]>,
}

fn not_space(s: &[u8]) -> IResult<&[u8], &[u8]> {
    is_not(" \t\r\n")(s)
}

fn setting_tag<'a>(input: &'a [u8]) -> IResult<&'a [u8], &[u8]> {
    alt((tag("set"), tag("add"), tag("append"), tag("prepend")))(input)
}

fn _parse_set<'a>(input: &'a [u8]) -> IResult<&'a [u8], (&[u8], &[u8], &[u8], &[u8], &[u8])> {
    let (input, (v, _, k, _, f, _,   e, _, _, _, val, _)) = tuple((setting_tag, space1, not_space, space1, not_space, space1, digit1, space1, digit1, crlf, take_until("\r\n"), crlf))(input)?;
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

pub fn parse(input: &[u8]) -> Result<Command<'_>, String> {
    match parse_raw_command(input){
        Ok((_input, cmd)) => {
            match cmd.verb.as_str() {
                "get" => Ok(Command::MGet {key: cmd.args[0]}),
                "set" => Ok(Command::MSet {key: cmd.args[0], flags: cmd.args[1], ttl: bytes_to_u64(cmd.args[2]), value: cmd.args[3]}),
                "add" => Ok(Command::MAdd {key: cmd.args[0], flags: cmd.args[1], ttl: bytes_to_u64(cmd.args[2]), value: cmd.args[3]}),
                "append" => Ok(Command::MAppend {key: cmd.args[0], flags: cmd.args[1], ttl: bytes_to_u64(cmd.args[2]), value: cmd.args[3]}),
                "prepend" => Ok(Command::MPrepend {key: cmd.args[0], flags: cmd.args[1], ttl: bytes_to_u64(cmd.args[2]), value: cmd.args[3]}),
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
        let result = parse(b"set myKey 0 60 19\r\nthe value to store\r\n");
        assert_eq!(result.unwrap(), Command::MSet {key: b"myKey", flags: b"0", ttl: 60u64, value: b"the value to store"});
    }

    #[test]
    fn parse_for_add() {
        let result = parse(b"add myKey 0 60 19\r\nthe value to store\r\n");
        assert_eq!(result.unwrap(), Command::MAdd {key: b"myKey", flags: b"0", ttl: 60u64, value: b"the value to store"});
    }
}
