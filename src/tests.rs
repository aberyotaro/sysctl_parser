#[cfg(test)]
mod tests {
    use {
        std::io::Write,
        tempfile::NamedTempFile,
        std::fs::File,
    };
    use crate::parse;

    #[test]
    fn test_parse() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "#").unwrap();
        writeln!(file, "kernel.domainname = example.com").unwrap();
        writeln!(file, "; A value containing a space is written to the sysctl.").unwrap();
        writeln!(file, "kernel.modprobe = /sbin/mod probe").unwrap();
        writeln!(file, "token1 = value1 # this is comment").unwrap();
        writeln!(file, "token2 = value2 ; this is comment").unwrap();
        writeln!(file, "token3 = value3 ;# this is comment").unwrap();
        writeln!(file, "token4 = value4 #; this is comment").unwrap();

        let map = parse(File::open(file.path()).unwrap(), None).unwrap();
        assert_eq!(map.get("kernel.domainname"), Some(&"example.com".to_string()));
        assert_eq!(map.get("kernel.modprobe"), Some(&"/sbin/mod probe".to_string()));
        assert_eq!(map.get("token1"), Some(&"value1".to_string()));
        assert_eq!(map.get("token2"), Some(&"value2".to_string()));
        assert_eq!(map.get("token3"), Some(&"value3".to_string()));
        assert_eq!(map.len(), 6);
    }

    #[test]
    fn test_parse_with_scheme() {
        let mut conf_file = NamedTempFile::new().unwrap();
        writeln!(conf_file, "kernel.domainname = example.com").unwrap();
        writeln!(conf_file, "kernel.modprobe = /sbin/mod probe").unwrap();
        writeln!(conf_file, "param_string = this is string").unwrap();
        writeln!(conf_file, "param_bool = true").unwrap();
        writeln!(conf_file, "param_int = -12345").unwrap();
        writeln!(conf_file, "param_uint = 12345").unwrap();
        writeln!(conf_file, "param_float = 0.12345").unwrap();

        let mut scheme_file = NamedTempFile::new().unwrap();
        writeln!(scheme_file, "kernel.domainname -> string").unwrap();
        writeln!(scheme_file, "kernel.modprobe -> string").unwrap();
        writeln!(scheme_file, "param_string -> string").unwrap();
        writeln!(scheme_file, "param_bool -> bool").unwrap();
        writeln!(scheme_file, "param_int -> int").unwrap();
        writeln!(scheme_file, "param_uint -> uint").unwrap();
        writeln!(scheme_file, "param_float -> float").unwrap();

        let map = parse(
            File::open(conf_file.path()).unwrap(),
            Some(File::open(scheme_file.path()).unwrap())
        ).unwrap();
        assert_eq!(map.get("kernel.domainname"), Some(&"example.com".to_string()));
        assert_eq!(map.get("kernel.modprobe"), Some(&"/sbin/mod probe".to_string()));
        assert_eq!(map.get("param_string"), Some(&"this is string".to_string()));
        assert_eq!(map.get("param_bool"), Some(&"true".to_string()));
        assert_eq!(map.get("param_int"), Some(&"-12345".to_string()));
        assert_eq!(map.get("param_uint"), Some(&"12345".to_string()));
        assert_eq!(map.get("param_float"), Some(&"0.12345".to_string()));
        assert_eq!(map.len(), 7);
    }
}
