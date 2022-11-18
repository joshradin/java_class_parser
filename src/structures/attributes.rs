//! Parsed attributes

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use crate::class::JavaClass;
use crate::constant_pool::parser::parse_attribute_info;
use crate::raw_java_class::RawAttributeInfo;
use crate::utility::match_as;
use crate::{ConstantPoolInfo, HasAttributes};
use crate::FullyQualifiedName;
use byteorder::ByteOrder;
use nom::bytes::complete::take;
use nom::combinator::{complete, flat_map, map, map_res};
use nom::multi::count;
use nom::number::complete::{be_u16, be_u32};
use nom::sequence::tuple;
use nom::{Finish, IResult};
use std::path::Path;

/// An attribute info piece. Can be parsed into usable data
#[derive(Debug)]
pub struct Attribute<'a> {
    attribute_name: &'a str,
    kind: AttributeKind<'a>,
}

/// The kind of attribute
#[derive(Debug)]
pub enum AttributeKind<'a> {
    SourceFile(&'a Path),
    Signature(&'a str),
    Code(Code<'a>),
    LineNumberTable(LineNumberTable),
    Deprecated,
    /// An unknown attribute
    Unknown(&'a [u8]),
}

impl<'a> Attribute<'a> {
    pub(crate) fn new(
        class: &'a JavaClass,
        attribute_name: &'a str,
        bytes: &'a [u8],
    ) -> Result<Self, ResolveAttributeError> {
        let error = || ResolveAttributeError::new(attribute_name);

        let kind: AttributeKind = match attribute_name {
            "SourceFile" => {
                let index = byteorder::BigEndian::read_u16(bytes);
                let utf8 = class.get_string(index).ok_or(error())?;
                AttributeKind::SourceFile(Path::new(utf8))
            }
            "Signature" => {
                let index = byteorder::BigEndian::read_u16(bytes);
                let utf8 = class.get_string(index).ok_or(error())?;
                AttributeKind::Signature(utf8)
            }
            "Code" => {
                let (_, code) = parse_code_attr(bytes, class).finish().unwrap();
                AttributeKind::Code(code)
            }
            "LineNumberTable" => {
                let parser = |bytes| -> IResult<&[u8], Vec<(u16, u16)>> {
                    flat_map(be_u16, |length: u16| {
                        count(tuple((be_u16, be_u16)), length as usize)
                    })(bytes)
                };
                let (_, lines) = parser(bytes)
                    .finish().unwrap();
                AttributeKind::LineNumberTable(LineNumberTable { line_number_table: lines.into_boxed_slice() })
            }
            "Deprecated" => {
                AttributeKind::Deprecated
            }
            _ => AttributeKind::Unknown(bytes),
        };
        Ok(Self {
            attribute_name,
            kind,
        })
    }

    /// Gets the name of the attribute
    pub fn attribute_name(&self) -> &'a str {
        self.attribute_name
    }

    /// Gets the attribute kind.
    ///
    /// Known attributes are defined in section [§4.7](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.7)
    /// of the JVM specification.
    ///
    /// If the attribute kind is not known (based on the attribute name), the
    /// [unknown](AttributeKind::Unknown) member is returned).
    pub fn kind(&self) -> &AttributeKind<'a> {
        &self.kind
    }
}

/// An error occurred while resolving an attribute.
#[derive(Debug, thiserror::Error)]
#[error("An error occurred while resolving attribute {0}")]
pub struct ResolveAttributeError(String);
impl ResolveAttributeError {
    pub(crate) fn new<S: AsRef<str>>(string: S) -> Self {
        Self(string.as_ref().to_string())
    }
}

/// The code attribute
pub struct Code<'a> {
    class: &'a JavaClass,
    max_stack: u16,
    max_locals: u16,
    code: &'a [u8],
    exception_table: Box<[Exception<'a>]>,
    attributes: Box<[RawAttributeInfo]>,
}

impl HasAttributes for Code<'_> {
    type Iter<'a> = <Vec<Attribute<'a>> as IntoIterator>::IntoIter where Self: 'a;

    fn attributes<'a>(&'a self) -> Self::Iter<'a> {
        self.attributes.iter()
            .map(|raw| self.class.create_attribute(
                raw.attribute_name_index, &raw.info
            ).unwrap())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Debug for Code<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Code")
            .field("max_stack", &self.max_stack)
            .field("max_locals", &self.max_locals)
            .field("code", &self.code)
            .field("exception_table", &self.exception_table)
            .field("attributes", &self.attributes().collect::<Vec<_>>())
            .finish()
    }
}

/// Each entry in the exception table describes one exception handler in the code array.
#[derive(Debug)]
pub struct Exception<'a> {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: Option<FullyQualifiedName<'a>>,
}




fn parse_code_attr<'a>(info: &'a [u8], class: &'a JavaClass) -> IResult<&'a [u8], Code<'a>> {
    map(
        complete(tuple((
            be_u16,
            be_u16,
            flat_map(be_u32, |code_length: u32| take(code_length)),
            flat_map(be_u16, |exception_table_length: u16| {
                count(
                    |b| parse_exception(b, class),
                    exception_table_length as usize,
                )
            }),
            flat_map(be_u16, |attribute_length: u16| {
                count(parse_attribute_info, attribute_length as usize)
            }),
        ))),
        |(max_stack, max_locals, code, exception_table, attributes)| Code {
            class,
            max_stack,
            max_locals,
            code,
            exception_table: exception_table.into_boxed_slice(),
            attributes: attributes.into_boxed_slice(),
        },
    )(info)
}

fn parse_exception<'a>(bytes: &'a [u8], class: &'a JavaClass) -> IResult<&'a [u8], Exception<'a>> {
    map(
        tuple((be_u16, be_u16, be_u16, be_u16)),
        |(start_pc, end_pc, handler_pc, catch_type_index)| Exception {
            start_pc,
            end_pc,
            handler_pc,
            catch_type: if catch_type_index == 0 {
                None
            } else {
                class
                    .get_at_index(catch_type_index)
                    .and_then(|info| match_as!(utf; ConstantPoolInfo::Utf8(utf) = info))
                    .map(|utf8| FullyQualifiedName::new(utf8.as_ref()))
            },
        },
    )(bytes)
}

pub struct LineNumberTable {
    line_number_table: Box<[(u16, u16)]>
}

impl LineNumberTable {
    /// Converts a byte in the code to a line number
    pub fn pc_to_line(&self, pc: u16) -> Option<u16> {
        let mut output = None;
        for &(start_pc, line_number) in &self.line_number_table[..] {
            if pc > start_pc {
                break;
            } else {
                output = Some(line_number);
            }
        }
        output
    }
}

impl Debug for LineNumberTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.line_number_table
            .iter()
            .copied()
            .collect::<HashMap<_, _>>()
            .fmt(f)
    }
}