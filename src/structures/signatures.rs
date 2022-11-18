use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_while};
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::combinator::{eof, map};
use nom::error::ErrorKind;
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A signature
#[derive(Debug, PartialEq)]
pub enum Signature<'a> {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Void,
    FullyQualifiedClass(&'a str),
    Array(Box<Signature<'a>>),
    Method {
        args: Box<[Signature<'a>]>,
        ret_type: Box<Signature<'a>>,
    },
}

impl<'a> Signature<'a> {
    /// emits this signature as JNI
    fn jni(&self) -> String {
        match self {
            Signature::Boolean => "Z".to_string(),
            Signature::Byte => "B".to_string(),
            Signature::Char => "C".to_string(),
            Signature::Short => "S".to_string(),
            Signature::Int => "I".to_string(),
            Signature::Long => "J".to_string(),
            Signature::Float => "F".to_string(),
            Signature::Double => "D".to_string(),
            Signature::FullyQualifiedClass(f) => {
                format!("L{f};")
            }
            Signature::Array(array) => {
                format!("[{}", array.jni())
            }
            Signature::Method { args, ret_type } => {
                format!(
                    "({}){}",
                    args.iter().map(|s| s.jni()).collect::<String>(),
                    ret_type.jni()
                )
            }
            Signature::Void => "V".to_string(),
        }
    }

    pub fn from_str(str: &'a str) -> Result<Self, nom::Err<nom::error::Error<String>>> {
        let (bytes, parsed) =
            parse_signature(str).map_err(|e: nom::Err<nom::error::Error<&str>>| e.to_owned())?;
        eof(bytes).map_err(|e: nom::Err<nom::error::Error<&str>>| e.to_owned())?;
        Ok(parsed)
    }
}

impl Display for Signature<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Signature::Boolean => {
                write!(f, "boolean")
            }
            Signature::Byte => {
                write!(f, "byte")
            }
            Signature::Char => {
                write!(f, "char")
            }
            Signature::Short => {
                write!(f, "short")
            }
            Signature::Int => {
                write!(f, "int")
            }
            Signature::Long => {
                write!(f, "long")
            }
            Signature::Float => {
                write!(f, "float")
            }
            Signature::Double => {
                write!(f, "double")
            }
            Signature::FullyQualifiedClass(fqc) => {
                write!(f, "{}", fqc)
            }
            Signature::Array(array) => {
                write!(f, "{}[]", array)
            }
            Signature::Method { args, ret_type } => {
                write!(
                    f,
                    "{} ({})",
                    ret_type,
                    args.iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Signature::Void => {
                write!(f, "void")
            }
        }
    }
}

fn parse_signature(string: &str) -> IResult<&str, Signature> {
    nom::branch::alt((
        map(tag("Z"), |_| Signature::Boolean),
        map(tag("B"), |_| Signature::Byte),
        map(tag("C"), |_| Signature::Char),
        map(tag("S"), |_| Signature::Short),
        map(tag("I"), |_| Signature::Int),
        map(tag("J"), |_| Signature::Long),
        map(tag("F"), |_| Signature::Float),
        map(tag("D"), |_| Signature::Double),
        map(tag("V"), |_| Signature::Void),
        map(
            delimited(tag("L"), take_till(|c| c == ';'), tag(";")),
            |chars| Signature::FullyQualifiedClass(chars),
        ),
        map(preceded(tag("["), parse_signature), |s| {
            Signature::Array(Box::new(s))
        }),
        map(
            tuple((
                delimited(tag("("), many0(parse_signature), tag(")")),
                parse_signature,
            )),
            |(args, ret_type)| Signature::Method {
                args: args.into_boxed_slice(),
                ret_type: Box::new(ret_type),
            },
        ),
    ))(string)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_jni_signature() {
        let jni = "(ZI)Ljava/lang/Object;";
        let parsed = Signature::from_str(jni).expect("couldn't parse");
        assert_eq!(
            parsed,
            Signature::Method {
                args: vec![Signature::Boolean, Signature::Int].into_boxed_slice(),
                ret_type: Box::new(Signature::FullyQualifiedClass("java/lang/Object"))
            }
        );
        assert_eq!(parsed.jni(), jni);
        assert_eq!(parsed.to_string(), "java/lang/Object (boolean, int)")
    }
}
