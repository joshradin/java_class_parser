//! The raw java class, a direct translation of the java [ClassFile structure][class_file]
//!
//! [class_file]: https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.1

use crate::constant_pool::{parser, ConstantPool};
use crate::error::Error;
use nom::combinator::eof;
use nom::error::ParseError;
use nom::number::complete::{be_u16, be_u32};
use nom::sequence::tuple;
use nom::{multi, IResult};

/// A raw java class file structure. All members have public access.
///
/// Defined by the [jvm spec](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.1).
#[derive(Debug, Clone)]
pub struct RawJavaClass {
    pub magic: u32,
    pub major: u16,
    pub minor: u16,
    pub constant_pool_count: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces_count: u16,
    pub interfaces: Box<[u16]>,
    pub fields_count: u16,
    pub fields: Box<[RawFieldInfo]>,
    pub methods_count: u16,
    pub methods: Box<[RawMethodInfo]>,
    pub attributes_count: u16,
    pub attributes: Box<[RawAttributeInfo]>,
}

/// The raw field info structure
#[derive(Debug, Default, Clone)]
pub struct RawFieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Box<[RawAttributeInfo]>,
}

/// The raw method info structure
#[derive(Debug, Default, Clone)]
pub struct RawMethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Box<[RawAttributeInfo]>,
}

/// The raw attribute info struct
#[derive(Debug, Default, Clone)]
pub struct RawAttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub info: Box<[u8]>,
}

/// Should parse the entire byte array to create a raw java class
pub fn parse_class_file_bytes(bytes: &[u8]) -> Result<RawJavaClass, Error> {
    fn inner<'a, E: ParseError<&'a [u8]>>(bytes: &'a [u8]) -> IResult<&'a [u8], RawJavaClass, E> {
        let mut tuple_parser = tuple((be_u32, be_u16, be_u16, be_u16));

        let (bytes, (magic, major, minor, constant_pool_count)) = tuple_parser(bytes)?;

        // for some reason, the constant pool contains n - 1 entries
        let (bytes, constant_pool) = parser::parse_constant_pool(constant_pool_count - 1)(bytes)?;

        let mut tuple_parser = tuple((be_u16, be_u16, be_u16, be_u16));
        let (bytes, (access_flags, this_class, super_class, interfaces_count)) =
            tuple_parser(bytes)?;
        let (bytes, interfaces) = multi::count(be_u16, interfaces_count as usize)(bytes)?;

        let (bytes, fields_count) = be_u16(bytes)?;
        let mut fields = vec![RawFieldInfo::default(); fields_count as usize];
        let (bytes, _) = multi::fill(parser::parse_field_info, &mut fields)(bytes)?;

        let (bytes, methods_count) = be_u16(bytes)?;
        let mut methods = vec![RawMethodInfo::default(); methods_count as usize];
        let (bytes, _) = multi::fill(parser::parse_method_info, &mut methods)(bytes)?;

        let (bytes, attributes_count) = be_u16(bytes)?;
        let mut attributes = vec![RawAttributeInfo::default(); attributes_count as usize];
        let (bytes, _) = multi::fill(parser::parse_attribute_info, &mut attributes)(bytes)?;

        let (bytes, _) = eof(bytes)?;

        Ok((
            bytes,
            RawJavaClass {
                magic,
                major,
                minor,
                constant_pool_count,
                constant_pool,
                access_flags,
                this_class,
                super_class,
                interfaces_count,
                interfaces: interfaces.into_boxed_slice(),
                fields_count,
                fields: fields.into_boxed_slice(),
                methods_count,
                methods: methods.into_boxed_slice(),
                attributes_count,
                attributes: attributes.into_boxed_slice(),
            },
        ))
    }

    inner::<nom::error::Error<_>>(bytes)
        .map(|(_, java)| java)
        .map_err(|e| Error::from(e))
}
