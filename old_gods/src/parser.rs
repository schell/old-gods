//! Parsing helpers for various Tiled parsing tasks.
pub use nom::branch::alt;
pub use nom::bytes::complete::{tag, take_till, take_while_m_n};
pub use nom::character::complete::{char, digit1, multispace0, multispace1};
use nom::combinator::map_res;
pub use nom::error::ErrorKind;
pub use nom::multi::separated_list;
pub use nom::number::complete::{be_u32, float};
pub use nom::sequence::tuple;
pub use nom::Err;
pub use nom::{AsChar, IResult, InputIter, InputTakeAtPosition, Slice};

use super::color::Color;

/// Parse a string
pub fn string(i: &str) -> IResult<&str, String> {
    let (i, _) = char('"')(i)?;
    let (i, n) = take_till(|c| c == '"')(i)?;
    let (i, _) = char('"')(i)?;
    Ok((i, n.to_string()))
}

/// Parse a tuple of 2 params.
pub fn params2<I, X, Y, A, B>(item1: X, item2: Y) -> impl Fn(I) -> IResult<I, (A, B)>
where
    X: Fn(I) -> IResult<I, A>,
    Y: Fn(I) -> IResult<I, B>,
    I: InputIter + InputTakeAtPosition + Clone + Slice<std::ops::RangeFrom<usize>>,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    <I as InputIter>::Item: AsChar + Clone,
{
    move |i: I| {
        let comma = tuple((multispace0, char(','), multispace0));
        let (i, _) = char('(')(i)?;
        let (i, _) = multispace0(i)?;
        let (i, a) = item1(i)?;
        let (i, _) = comma(i)?;
        let (i, b) = item2(i)?;
        let (i, _) = multispace0(i)?;
        let (i, _) = char(')')(i)?;
        Ok((i, (a, b)))
    }
}


/// Parse a vec
pub fn vec<I, G, A>(parse_item: &'static G) -> impl Fn(I) -> IResult<I, Vec<A>>
where
    G: Fn(I) -> IResult<I, A>,
    I: InputIter + InputTakeAtPosition + Clone + PartialEq + Slice<std::ops::RangeFrom<usize>>,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    <I as InputIter>::Item: AsChar + Clone,
{
    move |i: I| {
        let comma = tuple((multispace0, char(','), multispace0));
        let (i, _) = char('[')(i)?;
        let (i, v) = separated_list(comma, parse_item)(i)?;
        let (i, _) = char(']')(i)?;
        Ok((i, v))
    }
}


fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}


pub fn hex_color_3(input: &str) -> IResult<&str, (u8, u8, u8)> {
    let (input, _) = tag("#")(input)?;
    tuple((hex_primary, hex_primary, hex_primary))(input)
}


pub fn hex_color_4(input: &str) -> IResult<&str, (u8, u8, u8, u8)> {
    let (input, _) = tag("#")(input)?;
    tuple((hex_primary, hex_primary, hex_primary, hex_primary))(input)
}


pub fn hex_color_rgba(input: &str) -> IResult<&str, Color> {
    let (i, (a, r, g, b)) = hex_color_4(input)?;
    Ok((i, Color::rgba(r, g, b, a)))
}


pub fn hex_color_rgb(input: &str) -> IResult<&str, Color> {
    let (i, (r, g, b)) = hex_color_3(input)?;
    Ok((i, Color::rgb(r, g, b)))
}

pub fn hex_color(input: &str) -> IResult<&str, Color> {
    alt((hex_color_rgba, hex_color_rgb))(input)
}
