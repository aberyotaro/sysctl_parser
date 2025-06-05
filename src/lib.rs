use {
    std::collections::HashMap,
    std::io::BufRead,
};

pub fn parse(file: std::fs::File) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();

    for (idx, line_result) in std::io::BufReader::new(file).lines().enumerate() {
        let line = line_result.map_err(|e| format!("Failed to reading line {}: {}", idx + 1, e))?;

        if should_skip(&line) {
            continue;
        }

        let line = line.split('#').next().unwrap_or("");
        let v: Vec<&str> = line.split('=').collect();

        // validation
        if v.len() != 2 {
            if ignore_error(&line) {
                continue;
            }
            return Err(format!("Invalid line format at line {}: {}", idx + 1, line));
        }

        let key = v[0].trim();
        let value = v[1].trim();
        if key.contains(" ") || key.contains("\t") || key.contains("　") {
            if ignore_error(&line) {
                continue;
            }
            return Err(format!("Invalid space contains at line {}: {}", idx + 1, line));
        }

        map.insert(key.to_string(), value.to_string());
    }

    Ok(map)
}

fn ignore_error(str: &str) -> bool {
    str.starts_with('-')
}

fn should_skip(str: &str) -> bool {
    str.is_empty() || str.starts_with('#') || str.starts_with(';')
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "#").unwrap();
        writeln!(file, "kernel.domainname = example.com").unwrap();
        writeln!(file, "; A value containing a space is written to the sysctl.").unwrap();
        writeln!(file, "kernel.modprobe = /sbin/mod probe").unwrap();

        let map = parse(std::fs::File::open(file.path()).unwrap()).unwrap();
        assert_eq!(map.get("kernel.domainname"), Some(&"example.com".to_string()));
        assert_eq!(map.get("kernel.modprobe"), Some(&"/sbin/mod probe".to_string()));
        assert_eq!(map.len(), 2);
    }
}
