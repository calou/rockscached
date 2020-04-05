use nom;

use crate::command::Command;
use nom::{
    IResult,
    bytes::complete::{tag, take_until, is_not},
    sequence::tuple,
    branch::alt,
    character::complete::{crlf, space1, digit1},
};
use crate::byte_utils::{bytes_to_u64, bytes_to_u32};
use nom::multi::many1;

#[derive(PartialEq, Debug)]
struct RawCommand<'a> {
    pub verb: String,
    pub args: Vec<&'a [u8]>,
}

fn not_space(s: &[u8]) -> IResult<&[u8], &[u8]> {
    is_not(" \t\r\n")(s)
}

fn _parse_set<'a>(input: &'a [u8]) -> IResult<&'a [u8], (&[u8], &[u8], &[u8], &[u8], &[u8])> {
    let alt_tags = alt((tag("set"), tag("add"), tag("append"), tag("prepend")));
    let (input, (v, _, k, _, f, _, e, _, _, _, val, _)) = tuple((alt_tags, space1, not_space, space1, digit1, space1, digit1, space1, digit1, crlf, take_until("\r\n"), crlf))(input)?;
    Ok((input, (v, k, f, e, val)))
}

fn parse_set<'a>(input: &'a [u8]) -> IResult<&'a [u8], RawCommand<'_>> {
    match _parse_set(input) {
        Ok((input, (v, key, flags, expiration_timestamp, value))) => {
            Ok((input, RawCommand { verb: String::from_utf8(v.to_vec()).unwrap(), args: vec![key, flags, expiration_timestamp, value] }))
        }
        Err(e) => Result::Err(e)
    }
}

fn _parse_incr<'a>(input: &'a [u8]) -> IResult<&'a [u8], (&[u8], &[u8], &[u8])> {
    let alt_tags = alt((tag("incr"), tag("decr")));
    let (input, (v, _, k, _, val, _)) = tuple((alt_tags, space1, not_space, space1, digit1, crlf))(input)?;
    Ok((input, (v, k, val)))
}

fn parse_incr<'a>(input: &'a [u8]) -> IResult<&'a [u8], RawCommand<'_>> {
    match _parse_incr(input) {
        Ok((input, (v, key, value))) => {
            Ok((input, RawCommand { verb: String::from_utf8(v.to_vec()).unwrap(), args: vec![key, value] }))
        }
        Err(e) => Result::Err(e)
    }
}

fn _parse_delete<'a>(input: &'a [u8]) -> IResult<&'a [u8], (&[u8], &[u8])> {
    let (input, (v, _, k, _)) = tuple((tag("delete"), space1, not_space, crlf))(input)?;
    Ok((input, (v, k)))
}

fn space_and_key<'a>(input: &'a [u8]) -> IResult<&'a [u8], &[u8]> {
    let (input, (_, k)) = tuple((space1, not_space))(input)?;
    Ok((input, k))
}

fn _parse_get<'a>(input: &'a [u8]) -> IResult<&'a [u8], (&[u8], Vec<&[u8]>)> {
    let alt_tags = alt((tag("get"), tag("gets")));
    let (input, (v, k, _)) = tuple((alt_tags, many1(space_and_key), crlf))(input)?;
    Ok((input, (v, k)))
}

fn parse_get<'a>(input: &'a [u8]) -> IResult<&'a [u8], RawCommand<'_>> {
    match _parse_get(input) {
        Ok((input, (v, keys))) => {
            Ok((input, RawCommand { verb: String::from_utf8(v.to_vec()).unwrap(), args: keys }))
        }
        Err(e) => Result::Err(e)
    }
}

fn parse_delete<'a>(input: &'a [u8]) -> IResult<&'a [u8], RawCommand<'_>> {
    match _parse_delete(input) {
        Ok((input, (v, key))) => {
            Ok((input, RawCommand { verb: String::from_utf8(v.to_vec()).unwrap(), args: vec![key] }))
        }
        Err(e) => Result::Err(e)
    }
}

fn parse_raw_command<'a>(input: &'a [u8]) -> IResult<&'a [u8], RawCommand<'_>> {
    let (input, cmd) = alt((parse_get, parse_delete, parse_set, parse_incr))(input)?;
    Ok((input, cmd))
}

pub fn parse(input: &[u8]) -> Result<Command<'_>, String> {
    match parse_raw_command(input) {
        Ok((_input, cmd)) => {
            match cmd.verb.as_str() {
                "get" => Ok(Command::Get { keys: cmd.args }),
                "delete" => Ok(Command::Delete { key: cmd.args[0] }),
                "set" => Ok(Command::Set { key: cmd.args[0], flags: bytes_to_u32(cmd.args[1]), ttl: bytes_to_u64(cmd.args[2]), value: cmd.args[3] }),
                "add" => Ok(Command::Add { key: cmd.args[0], flags: bytes_to_u32(cmd.args[1]), ttl: bytes_to_u64(cmd.args[2]), value: cmd.args[3] }),
                "append" => Ok(Command::Append { key: cmd.args[0], flags: bytes_to_u32(cmd.args[1]), ttl: bytes_to_u64(cmd.args[2]), value: cmd.args[3] }),
                "prepend" => Ok(Command::Prepend { key: cmd.args[0], flags: bytes_to_u32(cmd.args[1]), ttl: bytes_to_u64(cmd.args[2]), value: cmd.args[3] }),
                "incr" => Ok(Command::Increment { key: cmd.args[0], value: bytes_to_u64(cmd.args[1]) }),
                "decr" => Ok(Command::Decrement { key: cmd.args[0], value: bytes_to_u64(cmd.args[1]) }),
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
        assert_eq!(result.unwrap(), Command::Get { keys: vec![b"myKey"] });
    }

    #[test]
    fn parse_for_multiget() {
        let result = parse(b"get k1 k2 k3 k4\r\n");
        assert_eq!(result.unwrap(), Command::Get { keys: vec![b"k1", b"k2", b"k3", b"k4"] });
    }

    #[test]
    fn parse_for_delete() {
        let result = parse(b"delete myKey\r\n");
        assert_eq!(result.unwrap(), Command::Delete { key: b"myKey" });
    }

    #[test]
    fn parse_for_set() {
        let result = parse(b"set myKey 0 60 19\r\nthe value to store\r\n");
        assert_eq!(result.unwrap(), Command::Set { key: b"myKey", flags: 0, ttl: 60u64, value: b"the value to store" });
    }

    #[test]
    fn parse_for_add() {
        let result = parse(b"add myKey 0 60 19\r\nthe value to store\r\n");
        assert_eq!(result.unwrap(), Command::Add { key: b"myKey", flags: 0, ttl: 60u64, value: b"the value to store" });
    }

    #[test]
    fn parse_for_incr() {
        let result = parse(b"incr myKey 1234\r\n");
        assert_eq!(result.unwrap(), Command::Increment { key: b"myKey", value: 1234 });
    }

    #[test]
    fn parse_for_decr() {
        let result = parse(b"decr myKey 1234\r\n");
        assert_eq!(result.unwrap(), Command::Decrement { key: b"myKey", value: 1234 });
    }
}
