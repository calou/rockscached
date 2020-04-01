extern crate nom;

use crate::command::{Command, MemcachedCommand, MemcachedCommandSet};
use crate::command::Command::MemcachedSet;
use nom::{
    IResult,
    bytes::complete::{tag, take_until, take_while, take_while1, is_not},
    combinator::map_res,
    number::complete::be_u64,
    sequence::tuple,
    character::complete::{crlf, space1},
};


/*
fn parse_set(input: &[u8]) -> IResult<&[u8], Command> {
    let value = take_until("\r\n");
    let (i, (_, _, k, _, f, _, e, _, v, _)) = tuple((
        tag("set"),
        space1,
        alphanumerical,
        space1,
        alphanumerical,
        space1,
        be_u64,
        space1,
        value,
        crlf
    ))(input)?;
    Ok((i, Command::MemcachedSet { key: k, flags: f, expiration_timestamp: e, value: v }))
}
*/
fn not_space(s: &[u8]) -> IResult<&[u8], &[u8]> {
    is_not(" \t\r\n")(s)
}

fn parse_set<'a>(input: &'a [u8]) -> IResult<&'a[u8], (&[u8], &[u8], u64, &[u8])> {
    let (input, _) = tag("set")(input)?;
    let (input, _) = space1(input)?;
    let (input, k) = not_space(input)?;
    let (input, _) = space1(input)?;
    let (input, f) = not_space(input)?;
    let (input, _) = space1(input)?;
    let (input, e) = be_u64(input)?;
    let (input, _) = space1(input)?;
    let (input, v) = take_until("\r\n")(input)?;
    let (input, _) = crlf(input)?;
    Ok((input, (k, f, e, v )))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_set_simple() {
        let (input, parts) = parse_set("set myKey 0 60 the value to store\r\n".as_bytes()).ok().unwrap();
        println!("{:?}", parts);
        assert_eq!(parts.0, b"myKey");
        assert_eq!(parts.1, b"0");
        assert_eq!(parts.2, 60u64);
        assert_eq!(parts.3,  b"the value to store");
    }
}