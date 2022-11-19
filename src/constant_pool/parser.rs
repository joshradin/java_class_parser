use crate::constant_pool::cfg::*;
use crate::constant_pool::values::{
    Class, FieldRef, InterfaceMethodRef, MethodRef, NameAndType, Utf8,
};
use crate::constant_pool::{ConstantPool, ConstantPoolInfo};


pub use crate::raw_java_class::{RawAttributeInfo, RawFieldInfo, RawJavaClass, RawMethodInfo};

use nom::bytes::complete::take;
use nom::combinator::{map};
use nom::error::ParseError;
use nom::multi::count;
use nom::number::complete::{be_u16, be_u32};
use nom::number::streaming::be_u8;
use nom::sequence::tuple;
use nom::IResult;
use nom::multi;

fn parse_data_info<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
) -> IResult<&'a [u8], (u16, u16, u16, u16, Box<[RawAttributeInfo]>), E> {
    tuple((be_u16, be_u16, be_u16, be_u16))(bytes).and_then(
        |(bytes, (access_flags, name, descriptor, attributes_count))| {
            map(
                count(parse_attribute_info, attributes_count as usize),
                |vector| {
                    (
                        access_flags,
                        name,
                        descriptor,
                        attributes_count,
                        vector.into_boxed_slice(),
                    )
                },
            )(bytes)
        },
    )
}

pub(crate) fn parse_field_info<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
) -> IResult<&'a [u8], RawFieldInfo, E> {
    let (rest, inner) = parse_data_info(bytes)?;
    let (access_flags, name_index, descriptor_index, attributes_count, attributes) = inner;
    Ok((
        rest,
        RawFieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attributes,
        },
    ))
}

pub(crate) fn parse_method_info<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
) -> IResult<&'a [u8], RawMethodInfo, E> {
    let (rest, inner) = parse_data_info(bytes)?;
    let (access_flags, name_index, descriptor_index, attributes_count, attributes) = inner;
    Ok((
        rest,
        RawMethodInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attributes,
        },
    ))
}

pub(crate) fn parse_attribute_info<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
) -> IResult<&'a [u8], RawAttributeInfo, E> {
    tuple((be_u16, be_u32))(bytes).and_then(|(bytes, (name_index, length))| {
        map(multi::count(be_u8, length as usize), |vector| {
            RawAttributeInfo {
                attribute_name_index: name_index,
                attribute_length: length,
                info: vector.into_boxed_slice(),
            }
        })(bytes)
    })
}

fn parse_constant_pool_info<'a, E: ParseError<&'a [u8]>>(
    bytes: &'a [u8],
) -> IResult<&'a [u8], ConstantPoolInfo, E> {
    let (bytes, tag) = if let (bytes, &[tag]) = take(1 as usize)(bytes)? {
        (bytes, tag)
    } else {
        unreachable!()
    };
    let parsed_ref_info = tuple((be_u16, be_u16));

    match tag {
        CLASS_TAG => {
            let (bytes, name_index) = be_u16(bytes)?;
            Ok((bytes, ConstantPoolInfo::Class(Class { name_index })))
        }
        FIELD_REF_TAG => map(parsed_ref_info, |(class_index, name_and_type_index)| {
            ConstantPoolInfo::FieldRef(FieldRef {
                class_index,
                name_and_type_index,
            })
        })(bytes),
        METHOD_REF_TAG => map(parsed_ref_info, |(class_index, name_and_type_index)| {
            ConstantPoolInfo::MethodRef(MethodRef {
                class_index,
                name_and_type_index,
            })
        })(bytes),
        INTERFACE_METHOD_REF_TAG => map(parsed_ref_info, |(class_index, name_and_type_index)| {
            ConstantPoolInfo::InterfaceMethodRef(InterfaceMethodRef {
                class_index,
                name_and_type_index,
            })
        })(bytes),
        STRING_TAG => {
            todo!()
        }
        INTEGER_TAG => {
            todo!()
        }
        FLOAT_TAG => {
            todo!()
        }
        LONG_TAG => {
            todo!()
        }
        DOUBLE_TAG => {
            todo!()
        }
        NAME_AND_TYPE_TAG => map(parsed_ref_info, |(name_index, descriptor_index)| {
            ConstantPoolInfo::NameAndType(NameAndType {
                name_index,
                descriptor_index,
            })
        })(bytes),
        UTF8_TAG => {
            let (bytes, length) = be_u16(bytes)?;
            let (bytes, char_bytes) = take(length)(bytes)?;
            let vector = Vec::from(char_bytes);
            Ok((
                bytes,
                ConstantPoolInfo::Utf8(Utf8 {
                    bytes: vector.into_boxed_slice(),
                }),
            ))
        }
        METHOD_HANDLE_TAG => {
            todo!()
        }
        METHOD_TYPE_TAG => {
            todo!()
        }
        INVOKE_DYNAMIC_TAG => {
            todo!()
        }
        _ => panic!("unknown tag: {:x}", tag),
    }
}

/// parses an entire constant pool of a predetermined length
pub fn parse_constant_pool<'a, E: ParseError<&'a [u8]>>(
    length: u16,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], ConstantPool, E> {
    nom::combinator::map(
        multi::count(parse_constant_pool_info, length as usize),
        |vec| ConstantPool::new(vec),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant_pool::cfg::UTF8_TAG;
    use crate::constant_pool::parser::parse_constant_pool_info;
    use crate::raw_java_class::parse_class_file_bytes;
    use crate::utility::match_as;
    use nom::Finish;
    use std::path::{PathBuf};
    use std::{env, fs};

    #[test]
    fn parse_utf8_constant_pool_info() {
        const CONSTANT: [u8; 6] = [UTF8_TAG, 0, 3, b'a', b'b', b'c'];
        let parsed = parse_constant_pool_info::<nom::error::Error<_>>(&CONSTANT)
            .finish()
            .expect("should be able to parse");
        let utf8 = match_as!(x; ( &[], ConstantPoolInfo::Utf8(x)) = parsed)
            .expect("should be a ut8 expression with no extra bytes");
        assert_eq!(utf8.to_string(), "abc");
    }

    #[test]
    fn parse_class() {
        let file = PathBuf::new()
            .join(env::var("OUT_DIR").unwrap())
            .join("java/build/classes/java/main/com/example/Square.class");
        let bytes =
            fs::read(&file).unwrap_or_else(|_| panic!("couldn't read bytes at path {:?} (exists = {})", file, file.exists()));

        let raw = parse_class_file_bytes(&bytes);
        match raw {
            Ok(raw) => {
                println!("raw = {raw:#?}");
            }
            Err(e) => {
                eprintln!("{e:#}");
                panic!()
            }
        }
    }
}