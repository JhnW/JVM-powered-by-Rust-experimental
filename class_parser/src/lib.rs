use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("Data stream error")]
    CannotRead(#[from] std::io::Error),
    #[error("Unable to parse data: `{0}`")]
    Parsing(String),
    #[error("Link element index error.")]
    Link,
    #[error("UTF-8 string parsing.")]
    Encoding(#[from] std::string::FromUtf8Error),
}

mod proxy {
    use std::rc::Rc;
    use class::const_pool::{ConstPoolType, NameAndTypeInfoStruct, Utf8Info, ComponentRef, ClassInfo};
    use super::*;

    #[derive(Debug, Copy, Clone)]
    pub struct Proxy(pub u16);


    #[derive(Debug, Copy, Clone)]
    pub struct DoubleProxy {
        pub class: ProxyToProxyClass,
        pub name_and_type: ProxyToProxyNameAndType,
    }

    #[derive(Debug, Copy, Clone)]
    pub struct ProxyToProxyNameAndType(pub u16);

    #[derive(Debug, Copy, Clone)]
    pub struct ProxyToProxyClass(pub u16);

    #[derive(Debug, Copy, Clone)]
    pub struct NameAndTypeProxy {
        pub name: Proxy,
        pub descriptor: Proxy,
    }

    #[derive(Debug, Copy, Clone)]
    pub struct FieldRefProxy(pub DoubleProxy);

    #[derive(Debug, Copy, Clone)]
    pub struct MethodRefProxy(pub DoubleProxy);

    #[derive(Debug, Copy, Clone)]
    pub struct InterfaceMethodRefProxy(pub DoubleProxy);

    #[derive(Debug, Copy, Clone)]
    pub struct StringProxy(pub Proxy);

    #[derive(Debug, Copy, Clone)]
    pub struct ClassProxy(pub Proxy);


    pub enum ProxyConstPoolType {
        Value(ConstPoolType),
        NameAndType(NameAndTypeProxy),
        FieldRef(FieldRefProxy),
        MethodRef(MethodRefProxy),
        InterfaceMethodRef(InterfaceMethodRefProxy),
        String(StringProxy),
        Class(ClassProxy),
    }

    pub trait ResolveProxy: Sized {
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError>;
    }

    impl ResolveProxy for ConstPoolType {
        fn resolve(&self, _: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            Ok(self.clone())
        }
    }

    #[inline(always)]
    fn resolve_simple_proxy(proxy: &Proxy, pool: &[ProxyConstPoolType]) -> Result<Utf8Info, DeserializationError> {
        if let ProxyConstPoolType::Value(ConstPoolType::Utf8(utf8)) =
            pool.get((proxy.0) as usize).ok_or(DeserializationError::Link)? {
            return Ok(utf8.clone());
        }
        Err(DeserializationError::Link)
    }

    impl ResolveProxy for StringProxy {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            Ok(ConstPoolType::String(resolve_simple_proxy(&self.0, pool)?))
        }
    }

    impl ResolveProxy for ClassProxy {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            if let ProxyConstPoolType::Value(ConstPoolType::Utf8(utf8)) =
                pool.get(self.0.0 as usize).ok_or(DeserializationError::Link)? {
                return Ok(ConstPoolType::Class(ClassInfo(utf8.clone())));
            }
            Err(DeserializationError::Link)
        }
    }

    impl ResolveProxy for NameAndTypeProxy {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            Ok(ConstPoolType::NameAndType(Rc::new(NameAndTypeInfoStruct {
                name: resolve_simple_proxy(&self.name, pool)?,
                descriptor: resolve_simple_proxy(&self.descriptor, pool)?,
            })))
        }
    }

    #[inline(always)]
    fn resolve_double_proxy(proxy: &DoubleProxy, pool: &[ProxyConstPoolType]) -> Result<ComponentRef, DeserializationError> {
        let class = if let ConstPoolType::Class(class) = proxy.class.resolve(pool)? {
            Ok(class)
        } else { Err(DeserializationError::Link) }?;
        let name_and_type = if let ConstPoolType::NameAndType(name_and_type) = proxy.name_and_type.resolve(pool)? {
            Ok(name_and_type)
        } else { Err(DeserializationError::Link) }?;
        Ok(ComponentRef {
            class,
            name_and_type,
        })
    }

    impl ResolveProxy for FieldRefProxy {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            Ok(ConstPoolType::Field(resolve_double_proxy(&self.0, pool)?))
        }
    }


    impl ResolveProxy for MethodRefProxy {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            Ok(ConstPoolType::MethodRef(resolve_double_proxy(&self.0, pool)?))
        }
    }

    impl ResolveProxy for InterfaceMethodRefProxy {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            Ok(ConstPoolType::InterfaceMethodRef(resolve_double_proxy(&self.0, pool)?))
        }
    }

    impl ResolveProxy for ProxyToProxyNameAndType {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            if let ProxyConstPoolType::NameAndType(proxy) =
                pool.get(self.0 as usize).ok_or(DeserializationError::Link)? {
                return proxy.resolve(pool);
            }
            Err(DeserializationError::Link)
        }
    }

    impl ResolveProxy for ProxyToProxyClass {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            if let ProxyConstPoolType::Class(proxy) =
                pool.get(self.0 as usize).ok_or(DeserializationError::Link)? {
                return proxy.resolve(pool);
            }
            Err(DeserializationError::Link)
        }
    }

    impl ResolveProxy for ProxyConstPoolType {
        #[inline(always)]
        fn resolve(&self, pool: &[ProxyConstPoolType]) -> Result<ConstPoolType, DeserializationError> {
            match self {
                ProxyConstPoolType::Value(value) => value.resolve(pool),
                ProxyConstPoolType::NameAndType(value) => value.resolve(pool),
                ProxyConstPoolType::FieldRef(value) => value.resolve(pool),
                ProxyConstPoolType::MethodRef(value) => value.resolve(pool),
                ProxyConstPoolType::InterfaceMethodRef(value) => value.resolve(pool),
                ProxyConstPoolType::String(value) => value.resolve(pool),
                ProxyConstPoolType::Class(value) => value.resolve(pool)
            }
        }
    }
}

pub mod deserialization {
    use super::*;
    use std::io::{Error, Read, Seek};
    use std::rc::Rc;
    use byteorder::{BigEndian, ReadBytesExt};
    use class::const_pool::ConstPoolType;
    use class::const_pool::ClassInfo;
    use class::const_pool::Utf8Info;
    use class::const_pool::{LongInfo, DoubleInfo, FloatInfo, IntInfo};
    use class::const_pool::ConstPoolType::Utf8;
    use super::proxy::*;
    use class::attributes::*;
    use class::components::*;
    use class::BitFlags;
    use class::Class;

    trait Deserializable: Sized {
        fn deserialize(cursor: impl Read + ReadBytesExt) -> Result<Self, DeserializationError>;
    }

    trait DeserializableLinked: Sized {
        fn deserialize_link(cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<Self, DeserializationError>;
    }

    trait DeserializableLinkedNamed: Sized {
        fn deserialize_link_named(name: String, cursor: impl Read + ReadBytesExt + Seek,
                                  pool: &[ConstPoolType]) -> Result<Self, DeserializationError>;
    }

    #[inline(always)]
    fn get_real_index(mut cursor: impl Read + ReadBytesExt) -> Result<u16, DeserializationError> {
        let index = cursor.read_u16::<BigEndian>()?;
        if index == 0 {
            Err(DeserializationError::Link)
        } else {
            Ok(index - 1)
        }
    }

    impl Deserializable for Utf8Info {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<Utf8Info, DeserializationError> {
            let mut data: Vec<u8> = vec![0; cursor.read_u16::<BigEndian>()? as usize];
            let _ = cursor.read(&mut data[..])?;
            Ok(Rc::new(String::from_utf8(data)?))
        }
    }

    impl Deserializable for IntInfo {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<IntInfo, DeserializationError> {
            Ok(cursor.read_i32::<BigEndian>()?)
        }
    }

    impl Deserializable for FloatInfo {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<FloatInfo, DeserializationError> {
            Ok(cursor.read_f32::<BigEndian>()?)
        }
    }

    impl Deserializable for LongInfo {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<LongInfo, DeserializationError> {
            Ok(cursor.read_i64::<BigEndian>()?)
        }
    }

    impl Deserializable for DoubleInfo {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<DoubleInfo, DeserializationError> {
            Ok(cursor.read_f64::<BigEndian>()?)
        }
    }

    impl Deserializable for NameAndTypeProxy {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<NameAndTypeProxy, DeserializationError> {
            Ok(NameAndTypeProxy {
                name: Proxy(get_real_index(&mut cursor)?),
                descriptor: Proxy(get_real_index(&mut cursor)?),
            })
        }
    }

    impl Deserializable for Proxy {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<Proxy, DeserializationError> {
            Ok(Proxy(get_real_index(&mut cursor)?))
        }
    }

    impl Deserializable for DoubleProxy {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<DoubleProxy, DeserializationError> {
            Ok(DoubleProxy {
                class: ProxyToProxyClass(get_real_index(&mut cursor)?),
                name_and_type: ProxyToProxyNameAndType(get_real_index(&mut cursor)?),
            })
        }
    }

    impl Deserializable for ProxyConstPoolType {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<ProxyConstPoolType, DeserializationError> {
            match cursor.read_u8()? {
                1 => Ok(ProxyConstPoolType::Value(Utf8(Utf8Info::deserialize(&mut cursor)?))),
                3 => Ok(ProxyConstPoolType::Value(ConstPoolType::Int(IntInfo::deserialize(&mut cursor)?))),
                4 => Ok(ProxyConstPoolType::Value(ConstPoolType::Float(FloatInfo::deserialize(&mut cursor)?))),
                5 => Ok(ProxyConstPoolType::Value(ConstPoolType::Long(LongInfo::deserialize(&mut cursor)?))),
                6 => Ok(ProxyConstPoolType::Value(ConstPoolType::Double(DoubleInfo::deserialize(&mut cursor)?))),
                7 => Ok(ProxyConstPoolType::Class(ClassProxy(Proxy::deserialize(&mut cursor)?))),
                8 => Ok(ProxyConstPoolType::String(StringProxy(Proxy::deserialize(&mut cursor)?))),
                9 => Ok(ProxyConstPoolType::FieldRef(FieldRefProxy(DoubleProxy::deserialize(&mut cursor)?))),
                10 => Ok(ProxyConstPoolType::MethodRef(MethodRefProxy(DoubleProxy::deserialize(&mut cursor)?))),
                11 => Ok(ProxyConstPoolType::InterfaceMethodRef(InterfaceMethodRefProxy(DoubleProxy::deserialize(&mut cursor)?))),
                12 => Ok(ProxyConstPoolType::NameAndType(NameAndTypeProxy::deserialize(&mut cursor)?)),
                unexpected => Err(DeserializationError::Parsing(format!("Invalid const pool type id: {unexpected}")))
            }
        }
    }

    impl Deserializable for Vec<ConstPoolType> {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<Vec<ConstPoolType>, DeserializationError> {
            let proxy = (0..(cursor.read_u16::<BigEndian>()? - 1) as usize)
                .map(|_| ProxyConstPoolType::deserialize(&mut cursor))
                .collect::<Result<Vec<ProxyConstPoolType>, _>>()?;
            let pool = proxy.iter()
                .map(|p| p.resolve(&proxy))
                .collect::<Result<Vec<ConstPoolType>, _>>()?;
            Ok(pool)
        }
    }

    #[inline(always)]
    fn find_const_pool_element(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<Option<&ConstPoolType>, DeserializationError> {
        let index = cursor.read_u16::<BigEndian>()? as usize;
        if index == 0 {
            Ok(None)
        } else {
            Ok(Some(pool.get(index - 1).ok_or(DeserializationError::Link)?))
        }
    }

    impl DeserializableLinked for Utf8Info {
        #[inline(always)]
        fn deserialize_link(cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<Utf8Info, DeserializationError> {
            match find_const_pool_element(cursor, pool)?.ok_or(DeserializationError::Link)? {
                Utf8(info) => Ok(info.clone()),
                _ => Err(DeserializationError::Link)
            }
        }
    }

    impl DeserializableLinked for ClassInfo {
        #[inline(always)]
        fn deserialize_link(cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<ClassInfo, DeserializationError> {
            match find_const_pool_element(cursor, pool)?.ok_or(DeserializationError::Link)? {
                ConstPoolType::Class(info) => Ok(info.clone()),
                _ => Err(DeserializationError::Link)
            }
        }
    }

    impl DeserializableLinked for ConstValueType {
        #[inline(always)]
        fn deserialize_link(cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<ConstValueType, DeserializationError> {
            match find_const_pool_element(cursor, pool)?.ok_or(DeserializationError::Link)? {
                ConstPoolType::Long(data) => Ok(ConstValueType::Long(*data)),
                ConstPoolType::Int(data) => Ok(ConstValueType::Int(*data)),
                ConstPoolType::Float(data) => Ok(ConstValueType::Float(*data)),
                ConstPoolType::Double(data) => Ok(ConstValueType::Double(*data)),
                ConstPoolType::String(data) => Ok(ConstValueType::String(data.clone())),
                _ => Err(DeserializationError::Link)
            }
        }
    }

    impl DeserializableLinked for ConstantValueAttribute {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<ConstantValueAttribute, DeserializationError> {
            let _ = cursor.read_u32::<BigEndian>()?;
            let value = ConstValueType::deserialize_link(&mut cursor, pool)?;
            Ok(ConstantValueAttribute {
                value
            })
        }
    }

    impl DeserializableLinked for ExceptionEntry {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<ExceptionEntry, DeserializationError> {
            let start_pc = cursor.read_u16::<BigEndian>()?;
            let end_pc = cursor.read_u16::<BigEndian>()?;
            let handler_pc = cursor.read_u16::<BigEndian>()?;
            let catch_type: Option<ClassInfo> = match find_const_pool_element(cursor, pool)? {
                Some(value) => Some(match value {
                    ConstPoolType::Class(info) => Ok(info.clone()),
                    _ => Err(DeserializationError::Link)
                }?),
                None => None
            };
            Ok(ExceptionEntry {
                start_pc,
                end_pc,
                handler_pc,
                catch_type,
            })
        }
    }

    impl DeserializableLinked for SourceFileAttribute {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<SourceFileAttribute, DeserializationError> {
            let _ = cursor.read_u32::<BigEndian>()?;
            let file = Utf8Info::deserialize_link(&mut cursor, pool)?;
            Ok(SourceFileAttribute {
                file,
            })
        }
    }

    impl Deserializable for UnknownAttribute {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<UnknownAttribute, DeserializationError> {
            let size = cursor.read_u32::<BigEndian>()?;
            let mut buffer = vec![0u8; size as usize];
            let _ = cursor.read_exact(&mut buffer)?;
            Ok(UnknownAttribute {
                size
            })
        }
    }

    impl Deserializable for SyntheticAttribute {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<SyntheticAttribute, DeserializationError> {
            let size = cursor.read_u32::<BigEndian>()?;
            if size != 0 {
                return Err(DeserializationError::Parsing("Synthetic attribute must be zero-sized".into()));
            }
            Ok(SyntheticAttribute {})
        }
    }

    impl DeserializableLinked for LocalVariableEntry {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<LocalVariableEntry, DeserializationError> {
            Ok(LocalVariableEntry {
                start_pc: cursor.read_u16::<BigEndian>()?,
                length: cursor.read_u16::<BigEndian>()?,
                name: Utf8Info::deserialize_link(&mut cursor, pool)?,
                descriptor: Utf8Info::deserialize_link(&mut cursor, pool)?,
                index: cursor.read_u16::<BigEndian>()?,
            })
        }
    }

    impl DeserializableLinked for LocalVariableTableAttribute {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<LocalVariableTableAttribute, DeserializationError> {
            let _ = cursor.read_u32::<BigEndian>()?;
            let length = cursor.read_u16::<BigEndian>()?;
            let variables = (0..length)
                .map(|_| LocalVariableEntry::deserialize_link(&mut cursor, pool))
                .collect::<Result<Vec<LocalVariableEntry>, DeserializationError>>()?;
            Ok(LocalVariableTableAttribute {
                variables
            })
        }
    }

    impl Deserializable for LineNumberEntry {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<LineNumberEntry, DeserializationError> {
            Ok(LineNumberEntry {
                start_pc: cursor.read_u16::<BigEndian>()?,
                line: cursor.read_u16::<BigEndian>()?,
            })
        }
    }

    impl Deserializable for LineNumberTableAttribute {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<LineNumberTableAttribute, DeserializationError> {
            let _ = cursor.read_u32::<BigEndian>()?;
            let length = cursor.read_u16::<BigEndian>()?;
            let lines = (0..length)
                .map(|_| LineNumberEntry::deserialize(&mut cursor))
                .collect::<Result<Vec<LineNumberEntry>, DeserializationError>>()?;
            Ok(LineNumberTableAttribute {
                lines
            })
        }
    }

    impl DeserializableLinked for ExceptionsAttribute {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<ExceptionsAttribute, DeserializationError> {
            let _ = cursor.read_u32::<BigEndian>()?;
            let length = cursor.read_u16::<BigEndian>()?;
            let exceptions_classes = (0..length)
                .map(|_| ClassInfo::deserialize_link(&mut cursor, pool))
                .collect::<Result<Vec<ClassInfo>, DeserializationError>>()?;
            Ok(ExceptionsAttribute {
                exceptions_classes
            })
        }
    }

    impl Deserializable for BitFlags<AccessSpecifier> {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<BitFlags<AccessSpecifier>, DeserializationError> {
            BitFlags::from_bits(cursor.read_u16::<BigEndian>()?)
                .map_err(|_| DeserializationError::Parsing("Unable to parse bit flag.".into()))
        }
    }

    impl Deserializable for BitFlags<ClassAccessSpecifier> {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<BitFlags<ClassAccessSpecifier>, DeserializationError> {
            BitFlags::from_bits(cursor.read_u16::<BigEndian>()?)
                .map_err(|_| DeserializationError::Parsing("Unable to parse bit flag.".into()))
        }
    }

    impl Deserializable for BitFlags<ClassAccess> {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<BitFlags<ClassAccess>, DeserializationError> {
            BitFlags::from_bits(cursor.read_u16::<BigEndian>()?)
                .map_err(|_| DeserializationError::Parsing("Unable to parse bit flag.".into()))
        }
    }

    impl DeserializableLinked for ClassEntry {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<ClassEntry, DeserializationError> {
            let inner_class_info: Option<ClassInfo> = ClassInfo::deserialize_link(&mut cursor, pool).ok();
            let outer_class_info: Option<ClassInfo> = ClassInfo::deserialize_link(&mut cursor, pool).ok();
            let name = Utf8Info::deserialize_link(&mut cursor, pool)?;
            let access: BitFlags<ClassAccessSpecifier> = BitFlags::deserialize(&mut cursor)?;
            Ok(ClassEntry {
                inner_class_info,
                outer_class_info,
                name,
                access,
            })
        }
    }

    impl DeserializableLinked for InnerClassesAttribute {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<InnerClassesAttribute, DeserializationError> {
            let _ = cursor.read_u32::<BigEndian>()?;
            let length = cursor.read_u16::<BigEndian>()?;
            let classes = (0..length)
                .map(|_| ClassEntry::deserialize_link(&mut cursor, pool))
                .collect::<Result<Vec<ClassEntry>, DeserializationError>>()?;
            Ok(InnerClassesAttribute {
                classes
            })
        }
    }

    impl DeserializableLinked for CodeAttribute {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<CodeAttribute, DeserializationError> {
            let _ = cursor.read_u32::<BigEndian>()?;
            let max_stack = cursor.read_u16::<BigEndian>()?;
            let max_local = cursor.read_u16::<BigEndian>()?;
            let code_length = cursor.read_u32::<BigEndian>()?;
            if code_length == 0 {
                return Err(DeserializationError::Link);
            }
            let code = (0..code_length)
                .map(|_| cursor.read_u8())
                .collect::<Result<Vec<u8>, Error>>()?;
            let exception_table_length = cursor.read_u16::<BigEndian>()?;
            let exceptions = (0..exception_table_length)
                .map(|_| ExceptionEntry::deserialize_link(&mut cursor, pool))
                .collect::<Result<Vec<ExceptionEntry>, DeserializationError>>()?;
            let attributes: Vec<CodeAttributes> = Vec::deserialize_link(&mut cursor, pool)?;
            Ok(CodeAttribute {
                max_stack,
                max_local,
                code,
                exceptions,
                attributes,
            })
        }
    }

    impl DeserializableLinked for Attribute {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<Attribute, DeserializationError> {
            let name = Utf8Info::deserialize_link(&mut cursor, pool)?;
            match name.as_str() {
                "Synthetic" => Ok(Attribute::Synthetic(SyntheticAttribute::deserialize(&mut cursor)?)),
                "SourceFile" => Ok(Attribute::SourceFile(SourceFileAttribute::deserialize_link(&mut cursor, pool)?)),
                "LineNumberTable" => Ok(Attribute::LineNumberTable(LineNumberTableAttribute::deserialize(&mut cursor)?)),
                "LocalVariableTable" => Ok(Attribute::LocalVariableTable(LocalVariableTableAttribute::deserialize_link(&mut cursor, pool)?)),
                "Deprecated" => Ok(Attribute::Deprecated(DeprecatedAttribute {})),
                "InnerClasses" => Ok(Attribute::InnerClasses(InnerClassesAttribute::deserialize_link(&mut cursor, pool)?)),
                "Exceptions" => Ok(Attribute::Exceptions(ExceptionsAttribute::deserialize_link(&mut cursor, pool)?)),
                "Code" => Ok(Attribute::Code(CodeAttribute::deserialize_link(&mut cursor, pool)?)),
                "ConstantValue" => Ok(Attribute::ConstantValue(ConstantValueAttribute::deserialize_link(&mut cursor, pool)?)),
                _ => Ok(Attribute::Unknown(UnknownAttribute::deserialize(&mut cursor)?))
            }
        }
    }

    impl DeserializableLinked for CodeAttributes {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<CodeAttributes, DeserializationError> {
            let name = Utf8Info::deserialize_link(&mut cursor, pool)?;
            match name.as_str() {
                "LineNumberTable" => Ok(CodeAttributes::LineNumberTable(LineNumberTableAttribute::deserialize(&mut cursor)?)),
                "LocalVariableTable" => Ok(CodeAttributes::LocalVariableTable(LocalVariableTableAttribute::deserialize_link(&mut cursor, pool)?)),
                _ => Ok(CodeAttributes::Unknown(UnknownAttribute::deserialize(&mut cursor)?))
            }
        }
    }

    impl DeserializableLinked for Vec<Attribute> {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<Vec<Attribute>, DeserializationError> {
            let attributes_count = cursor.read_u16::<BigEndian>()?;
            let attributes = (0..attributes_count)
                .map(|_| Attribute::deserialize_link(&mut cursor, pool))
                .collect::<Result<Vec<Attribute>, DeserializationError>>()?;
            Ok(attributes)
        }
    }

    impl DeserializableLinked for Vec<CodeAttributes> {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<Vec<CodeAttributes>, DeserializationError> {
            let attributes_count = cursor.read_u16::<BigEndian>()?;
            let attributes = (0..attributes_count)
                .map(|_| CodeAttributes::deserialize_link(&mut cursor, pool))
                .collect::<Result<Vec<CodeAttributes>, DeserializationError>>()?;
            Ok(attributes)
        }
    }

    impl DeserializableLinked for ComponentInfo {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<ComponentInfo, DeserializationError> {
            let access = BitFlags::deserialize(&mut cursor)?;
            let name = Utf8Info::deserialize_link(&mut cursor, pool)?;
            let descriptor = Utf8Info::deserialize_link(&mut cursor, pool)?;
            let attributes: Vec<Attribute> = Vec::deserialize_link(&mut cursor, pool)?;
            Ok(ComponentInfo {
                access,
                name,
                descriptor,
                attributes,
            })
        }
    }

    impl DeserializableLinked for Vec<ComponentInfo> {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<Vec<ComponentInfo>, DeserializationError> {
            let components_count = cursor.read_u16::<BigEndian>()?;
            let components = (0..components_count)
                .map(|_| ComponentInfo::deserialize_link(&mut cursor, pool))
                .collect::<Result<Vec<ComponentInfo>, DeserializationError>>()?;
            Ok(components)
        }
    }

    impl Deserializable for ClassVersion {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<ClassVersion, DeserializationError> {
            Ok(ClassVersion {
                minor: cursor.read_u16::<BigEndian>()?,
                major: cursor.read_u16::<BigEndian>()?,
            })
        }
    }

    impl DeserializableLinked for Vec<ClassInfo> {
        #[inline(always)]
        fn deserialize_link(mut cursor: impl Read + ReadBytesExt, pool: &[ConstPoolType]) -> Result<Vec<ClassInfo>, DeserializationError> {
            let count = cursor.read_u16::<BigEndian>()?;
            let classes = (0..count)
                .map(|_| ClassInfo::deserialize_link(&mut cursor, pool))
                .collect::<Result<Vec<ClassInfo>, DeserializationError>>()?;
            Ok(classes)
        }
    }

    impl Deserializable for Class {
        #[inline(always)]
        fn deserialize(mut cursor: impl Read + ReadBytesExt) -> Result<Class, DeserializationError> {
            let magick = cursor.read_u32::<BigEndian>()?;
            if magick != 0xCAFEBABE {
                return Err(DeserializationError::Parsing("Its not JVM class file.".into()));
            }
            let version = ClassVersion::deserialize(&mut cursor)?;
            let const_pool: Vec<ConstPoolType> = Vec::deserialize(&mut cursor)?;
            let access: BitFlags<ClassAccess> = BitFlags::deserialize(&mut cursor)?;
            let this_class = ClassInfo::deserialize_link(&mut cursor, &const_pool)?;
            let super_class = ClassInfo::deserialize_link(&mut cursor, &const_pool).ok();
            let interfaces: Vec<ClassInfo> = Vec::deserialize_link(&mut cursor, &const_pool)?;
            let fields: Vec<FieldInfo> = Vec::deserialize_link(&mut cursor, &const_pool)?;
            let methods: Vec<MethodInfo> = Vec::deserialize_link(&mut cursor, &const_pool)?;
            let attributes: Vec<Attribute> = Vec::deserialize_link(&mut cursor, &const_pool)?;
            Ok(Class {
                version,
                const_pool,
                access,
                this_class,
                super_class,
                interfaces,
                fields,
                methods,
                attributes,
            })
        }
    }

    pub fn deserializable_class(mut cursor: impl Read + ReadBytesExt) -> Result<Class, DeserializationError> {
        Class::deserialize(&mut cursor)
    }
}




