#![deny(warnings)]
pub use enumflags2::BitFlags;

pub mod const_pool {
    use std::rc::Rc;

    pub type Utf8Info = Rc<String>;
    pub type FloatInfo = f32;
    pub type DoubleInfo = f64;
    pub type IntInfo = i32;
    pub type LongInfo = i64;
    pub type StringInfo = Utf8Info;

    #[derive(Clone, Debug)]
    pub struct ClassInfo(pub Utf8Info);

    #[derive(Clone, Debug)]
    pub struct NameAndTypeInfoStruct {
        pub name: Utf8Info,
        pub descriptor: Utf8Info
    }
    pub type NameAndTypeInfo = Rc<NameAndTypeInfoStruct>;

    #[derive(Clone, Debug)]
    pub struct ComponentRef {
        pub class: ClassInfo,
        pub name_and_type: NameAndTypeInfo
    }
    pub type FieldRefInfo = ComponentRef;
    pub type MethodRefInfo = ComponentRef;
    pub type InterfaceMethodRefInfo = ComponentRef;

    #[derive(Clone, Debug)]
    pub enum ConstPoolType {
        Utf8(Utf8Info),
        Float(FloatInfo),
        Double(DoubleInfo),
        Int(IntInfo),
        Long(LongInfo),
        String(StringInfo),
        Class(ClassInfo),
        NameAndType(NameAndTypeInfo),
        Field(FieldRefInfo),
        MethodRef(MethodRefInfo),
        InterfaceMethodRef(InterfaceMethodRefInfo)
    }
}

pub mod attributes {
    use enumflags2::{bitflags, BitFlags};
    use crate::const_pool;

    #[derive(Debug)]
    pub enum ConstValueType {
        Float(const_pool::FloatInfo),
        Double(const_pool::DoubleInfo),
        Int(const_pool::IntInfo),
        Long(const_pool::LongInfo),
        String(const_pool::StringInfo)
    }

    #[derive(Debug)]
    pub struct ConstantValueAttribute {
        pub value: ConstValueType
    }

    #[derive(Debug)]
    pub struct ExceptionEntry {
        pub start_pc: u16,
        pub end_pc: u16,
        pub handler_pc: u16,
        pub catch_type: Option<const_pool::ClassInfo>
    }

    #[derive(Debug)]
    pub enum CodeAttributes {
        LineNumberTable(LineNumberTableAttribute),
        LocalVariableTable(LocalVariableTableAttribute),
        Unknown(UnknownAttribute)
    }

    #[derive(Debug)]
    pub struct CodeAttribute {
        pub max_stack: u16,
        pub max_local: u16,
        pub code: Vec<u8>,
        pub exceptions: Vec<ExceptionEntry>,
        pub attributes: Vec<CodeAttributes>
    }

    #[derive(Debug)]
    pub struct ExceptionsAttribute {
        pub exceptions_classes: Vec<const_pool::ClassInfo>
    }

    #[bitflags]
    #[repr(u16)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum ClassAccessSpecifier
    {
        Public = 0x0001,
        Private = 0x0002,
        Protected = 0x0004,
        Static = 0x0008,
        Final = 0x0010,
        Interface = 0x0040,
        Abstract = 0x0080
    }

    #[derive(Debug)]
    pub struct ClassEntry {
        pub inner_class_info: Option<const_pool::ClassInfo>,
        pub outer_class_info: Option<const_pool::ClassInfo>,
        pub name: const_pool::Utf8Info,
        pub access: BitFlags<ClassAccessSpecifier>
    }

    #[derive(Debug)]
    pub struct InnerClassesAttribute {
        pub classes: Vec<ClassEntry>
    }

    #[derive(Debug)]
    pub struct SourceFileAttribute {
        pub file: const_pool::Utf8Info
    }

    #[derive(Debug)]
    pub struct LineNumberEntry {
        pub start_pc: u16,
        pub line: u16
    }

    #[derive(Debug)]
    pub struct LineNumberTableAttribute {
        pub lines: Vec<LineNumberEntry>
    }

    #[derive(Debug)]
    pub struct LocalVariableEntry {
        pub start_pc: u16,
        pub length: u16,
        pub name: const_pool::Utf8Info,
        pub descriptor: const_pool::Utf8Info,
        pub index: u16
    }

    #[derive(Debug)]
    pub struct LocalVariableTableAttribute {
        pub variables: Vec<LocalVariableEntry>
    }

    #[derive(Debug)]
    pub struct DeprecatedAttribute {
    }

    #[derive(Debug)]
    pub struct UnknownAttribute {
        pub size: u32
    }

    #[derive(Debug)]
    pub struct SyntheticAttribute {
    }

    #[derive(Debug)]
    pub enum Attribute {
        Code(CodeAttribute),
        Exceptions(ExceptionsAttribute),
        InnerClasses(InnerClassesAttribute),
        SourceFile(SourceFileAttribute),
        LineNumberTable(LineNumberTableAttribute),
        LocalVariableTable(LocalVariableTableAttribute),
        Deprecated(DeprecatedAttribute),
        ConstantValue(ConstantValueAttribute),
        Synthetic(SyntheticAttribute),
        Unknown(UnknownAttribute)
    }

}

pub mod components {
    use enumflags2::{bitflags, BitFlags};
    use crate::const_pool;
    use crate::attributes::Attribute;

    #[bitflags]
    #[repr(u16)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum AccessSpecifier
    {
        Public = 0x0001,
        Private = 0x0002,
        Protected = 0x0004,
        Static = 0x0008,
        Final = 0x0010,
        Volatile = 0x0040,
        Transient = 0x0080
    }

    #[derive(Debug)]
    pub struct ComponentInfo {
        pub access: BitFlags<AccessSpecifier>,
        pub name: const_pool::Utf8Info,
        pub descriptor: const_pool::Utf8Info,
        pub attributes: Vec<Attribute>
    }

    pub type Interface = const_pool::ClassInfo;
    pub type FieldInfo = ComponentInfo;
    pub type MethodInfo = ComponentInfo;

    #[derive(Debug)]
    pub struct ClassVersion {
        pub minor: u16,
        pub major: u16
    }

    #[bitflags]
    #[repr(u16)]
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum ClassAccess {
        Public = 0x0001,
        Final = 0x0010,
        Super = 0x0020,
        Interface = 0x0200,
        Abstract = 0x0400
    }
}

#[derive(Debug)]
pub struct Class {
    pub version: components::ClassVersion,
    pub const_pool: Vec<const_pool::ConstPoolType>,
    pub access: BitFlags<components::ClassAccess>,
    pub this_class: const_pool::ClassInfo,
    pub super_class: Option<const_pool::ClassInfo>,
    pub interfaces: Vec<components::Interface>,
    pub fields: Vec<components::FieldInfo>,
    pub methods: Vec<components::MethodInfo>,
    pub attributes: Vec<attributes::Attribute>
}


