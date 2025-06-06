use {
    std::fs::File,
    std::io::BufReader,
    std::collections::HashMap,
    std::io::BufRead,
};

pub fn parse(conf: File, scheme: Option<File>) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for (idx, line_result) in BufReader::new(conf).lines().enumerate() {
        let line = line_result.map_err(|e| format!("Failed to reading line {}: {}", idx + 1, e))?;

        if should_skip(&line) {
            continue;
        }

        let kv_str = retrieve_key_value_str(&line);
        let v: Vec<&str> = kv_str.split('=').collect();

        // validation
        if v.len() != 2 {
            if ignore_error(&line) {
                continue;
            }
            return Err(format!("Invalid conf line format at line {}: {}", idx + 1, line));
        }

        let key = v[0].trim();
        let value = v[1].trim();
        if key.contains(" ") || key.contains("\t") || key.contains("　") {
            if ignore_error(&line) {
                continue;
            }
            return Err(format!("Invalid conf space contains at line {}: {}", idx + 1, line));
        }

        map.insert(key.to_string(), value.to_string());
    }

    // verification by scheme
    if let Some(scheme_file) = scheme {
        for (idx, line_result) in BufReader::new(scheme_file).lines().enumerate() {
            let line = line_result.map_err(|e| format!("Failed to reading scheme line {}: {}", idx + 1, e))?;

            if should_skip(&line) {
                continue;
            }

            let kv_str = retrieve_key_value_str(&line);
            let v: Vec<&str> = kv_str.split("->").collect();

            if v.len() != 2 {
                return Err(format!("Invalid scheme line format at line {}: {}", idx + 1, line));
            }

            let scheme_key = v[0].trim();
            let scheme_value = v[1].trim();

            let conf_val = map.get(scheme_key);
            if let Some(v) = conf_val {
                if !validate_type(scheme_value, v) {
                    return Err(format!("Invalid scheme line format at line {}: {}", idx + 1, line));
                }
            }
        }
    }

    Ok(map)
}

fn validate_type(type_str: &str, value: &str) -> bool {
    match type_str {
        "string" => value.parse::<String>().is_ok(),
        "bool" => value.parse::<bool>().is_ok(),
        "int" => value.parse::<i32>().is_ok(),
        "uint" => value.parse::<u32>().is_ok(),
        "float" => value.parse::<f64>().is_ok(),
        _ => false,
    }
}

fn retrieve_key_value_str(line: &str) -> &str {
    if let Some(kv_str) = line.split('#').next() {
        return kv_str.split(';').next().unwrap_or("");
    }
    line.split(';').next().unwrap_or("")
}

fn ignore_error(str: &str) -> bool {
    str.starts_with('-')
}

fn should_skip(str: &str) -> bool {
    str.is_empty() || str.starts_with('#') || str.starts_with(';')
}


#[cfg(test)]
mod tests {
    use {
        super::*,
        std::io::Write,
        tempfile::NamedTempFile,
    };

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
