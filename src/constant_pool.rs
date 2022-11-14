//! contains the raw definitions for the constant pool

use crate::error::{Error, ErrorKind};
use nom::bytes;
use nom::bytes::complete::*;
use nom::number::complete::be_u16;
use std::ops::Index;
use values::{
    Class, Double, FieldRef, Float, Integer, InterfaceMethodRef, InvokeDynamic, Long, MethodHandle,
    MethodRef, MethodType, NameAndType, Utf8,
};

pub mod parser;
pub mod values;

/// Config values
pub mod cfg {
    pub const CLASS_TAG: u8 = 7;
    pub const FIELD_REF_TAG: u8 = 9;
    pub const METHOD_REF_TAG: u8 = 10;
    pub const INTERFACE_METHOD_REF_TAG: u8 = 11;
    pub const STRING_TAG: u8 = 8;
    pub const INTEGER_TAG: u8 = 3;
    pub const FLOAT_TAG: u8 = 4;
    pub const LONG_TAG: u8 = 5;
    pub const DOUBLE_TAG: u8 = 6;
    pub const NAME_AND_TYPE_TAG: u8 = 12;
    pub const UTF8_TAG: u8 = 1;
    pub const METHOD_HANDLE_TAG: u8 = 15;
    pub const METHOD_TYPE_TAG: u8 = 16;
    pub const INVOKE_DYNAMIC_TAG: u8 = 18;
}

/// The `cp_info` structure, represents in a constant
#[derive(Debug, Clone)]
pub enum ConstantPoolInfo {
    Class(Class),
    FieldRef(FieldRef),
    MethodRef(MethodRef),
    InterfaceMethodRef(InterfaceMethodRef),
    String(values::StringValue),
    Integer(Integer),
    Float(Float),
    Long(Long),
    Double(Double),
    NameAndType(NameAndType),
    Utf8(Utf8),
    MethodHandle(MethodHandle),
    MethodType(MethodType),
    InvokeDynamic(InvokeDynamic),
}

/// The constant pool contains an array of constants
#[derive(Debug, Clone)]
pub struct ConstantPool {
    pool: Vec<ConstantPoolInfo>,
}

impl ConstantPool {
    /// Creates a new constant pool from an iterator
    pub(crate) fn new<I: IntoIterator<Item = ConstantPoolInfo>>(pool: I) -> Self {
        Self {
            pool: pool.into_iter().collect(),
        }
    }

    /// Constant pools are accessed using u16 values.
    pub fn get(&self, index: u16) -> Option<&ConstantPoolInfo> {
        self.pool.get(index as usize - 1)
    }
}

impl Index<u16> for ConstantPool {
    type Output = ConstantPoolInfo;

    fn index(&self, index: u16) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}
