use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{line_ending, space0},
    combinator::{opt},
    multi::{many0},
    IResult,
    Finish,
    branch::alt,
};
use std::collections::BTreeMap;
use std::path::Path;


pub type Properties<'a> = BTreeMap<&'a str, Option<&'a str>>;

#[derive(Debug)]
// if serde is enabled, this should be #[derive(Serialize, Deserialize)], we use the macro
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dist<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub pathname: &'a Path,

    #[cfg_attr(feature = "serde", serde(borrow))]
    pub provides: Option<Properties<'a>>,

    #[cfg_attr(feature = "serde", serde(borrow))]
    pub requirements: Option<Properties<'a>>,
}

pub type Dists<'a> = BTreeMap<&'a str, Dist<'a>>;

pub fn parse(input : &str) -> Option<Dists> {
    let (s, dists) = file(input).finish().ok()?;
    eprintln!("s: {:?}", s);

    Some(dists)
}

fn file(input: &str) -> IResult<&str, Dists> {
    let (input, _) = header(input)?;
    dists(input)
}

fn header(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("# carton snapshot format: version 1.0\n")(input)?;
    let (input, _) = tag("DISTRIBUTIONS\n")(input)?;
    Ok((input, ()))
}

fn dists(input: &str) -> IResult<&str, Dists> {
    let (input, dists) = many0(dist)(input)?;
    Ok((input, dists.into_iter().collect()))
}

fn dist(input: &str) -> IResult<&str, (&str, Dist)> {
    let (input, _) = level_2(input)?;
    // nonspace to end of line
    let (input, name) = take_while1(|c: char| !c.is_whitespace())(input)?;
    let (input, _) = line_ending(input)?;
    let (input, pathname) = pathname(input)?;
    let (input, provides) = opt(provides)(input)?;
    let (input, requirements) = opt(requirements)(input)?;
    Ok((
        input,
        (
            name,
            Dist {
                pathname,
                provides,
                requirements,
            },
        ),
    ))
}



fn level_2(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("  ")(input)?;
    Ok((input, ()))
}

fn level_4(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("    ")(input)?;
    Ok((input, ()))
}

fn level_6(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("      ")(input)?;
    Ok((input, ()))
}

// pathname: J/JP/JPEACOCK/version-0.9912.tar.gz
fn pathname(input: &str) -> IResult<&str, &Path> {
    let (input, _) = level_4(input)?;
    let (input, _) = tag("pathname: ")(input)?;
    let (input, pathname) = take_until("\n")(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, Path::new(pathname)))
}

fn provides(input: &str) -> IResult<&str, Properties> {
    let (input, _) = level_4(input)?;
    let (input, _) = tag("provides:\n")(input)?;
    let (input, properties) = properties(input)?;
    Ok((input, properties))
}

fn requirements(input: &str) -> IResult<&str, Properties<'_>> {
    let (input, _) = level_4(input)?;
    let (input, _) = tag("requirements:\n")(input)?;
    let (input, properties) = properties(input)?;
    Ok((input, properties))
}

fn properties(input: &str) -> IResult<&str, BTreeMap<&str, Option<&str>>> {
    let (input, properties) = many0(property)(input)?;
    Ok((input, properties.into_iter().collect()))
}

// original regex was qr/^\s{6}([0-9A-Za-z_:]+) ([v0-9\._,=\!<>\s]+|undef)/,

fn undef(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = tag("undef")(input)?;
    Ok((input, None))
}

fn version(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = space0(input)?;
    let (input, version) = take_while1(|c: char| {
        c.is_numeric()
            || c == 'v'
            || c == '.'
            || c == '_'
            || c == ','
            || c == '='
            || c == '!'
            || c == '<'
            || c == '>'
    })(input)?;
    Ok((input, Some(version)))
}

fn property(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    let (input, _) = level_6(input)?;
    let (input, key) = take_while1(|c: char| c.is_alphanumeric() || c == ':' || c == '_')(input)?;
    let (input, _) = space0(input)?;
    let (input, value) = alt((version, undef))(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, (key, value)))
}



