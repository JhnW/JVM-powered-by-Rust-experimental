#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use class_parser::deserialization::deserializable_class;
    use std::path::PathBuf;
    use class::components::ClassAccess;
    //use class::const_pool::ConstPoolType::Class;

    #[test]
    fn load_simple_class_file() {
        let f = File::open(PathBuf::from(env!("TEST_RESOURCES_PATH")).join("HelloWorld.class")).unwrap();
        let mut cursor = BufReader::new(f);
        let class = deserializable_class(&mut cursor).unwrap();
        assert_eq!(class.version.major, 55);
        assert_eq!(class.version.minor, 0);
        assert_eq!(class.super_class.as_ref().unwrap().0.as_str(), "java/lang/Object");
        assert_eq!(class.this_class.0.as_str(), "HelloWorld");
        assert_eq!(class.const_pool.len(), 28);
        assert_eq!(class.methods.len(), 2);
        assert_eq!(class.fields.len(), 0);
        assert_eq!(class.interfaces.len(), 0);
        assert_eq!(class.attributes.len(), 1);
        assert_eq!(class.access, ClassAccess::Public | ClassAccess::Super);
    }
}

