mod constant;
mod tests;

use {
    std::fs::File,
    std::io::BufReader,
    std::collections::HashMap,
    std::io::BufRead,
};

pub fn parse(conf: File, scheme: Option<File>) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for (idx, line_result) in BufReader::new(conf).lines().enumerate() {
        let line = line_result.map_err(|e| format!("[conf]: Failed to reading line {}: {}", idx + 1, e))?;

        if skip_line(&line) {
            continue;
        }

        let kv_str = retrieve_key_value_str(&line);
        let kv: Vec<&str> = kv_str.split('=').collect();
        let k = kv[0].trim();
        let v = kv[1].trim();

        // validation
        {
            if kv.len() != 2 {
                if ignore_error(&line) {
                    continue;
                }
                return Err(format!("[conf]: Invalid format at line {}: {}", idx + 1, line));
            }
            if v.len() > constant::CONF_MAX_VALUE_LENGTH {
                if ignore_error(&line) {
                    continue;
                }
                return Err(format!("[conf]: Exceeds maximum length of {} at line {}: {}", constant::CONF_MAX_VALUE_LENGTH, idx + 1, line));
            }
            if k.contains(" ") || k.contains("\t") || k.contains("　") {
                if ignore_error(&line) {
                    continue;
                }
                return Err(format!("[conf]: Invalid space contains at line {}: {}", idx + 1, line));
            }
        }

        map.insert(k.to_string(), v.to_string());
    }

    // verification by scheme
    if let Some(scheme_file) = scheme {
        for (idx, line_result) in BufReader::new(scheme_file).lines().enumerate() {
            let line = line_result.map_err(|e| format!("[scheme]: Failed to reading scheme line {}: {}", idx + 1, e))?;

            if skip_line(&line) {
                continue;
            }

            let kv_str = retrieve_key_value_str(&line);
            let v: Vec<&str> = kv_str.split("->").collect();

            if v.len() != 2 {
                return Err(format!("[scheme]: Invalid format at line {}: {}", idx + 1, line));
            }

            let scheme_key = v[0].trim();
            let scheme_value = v[1].trim();

            let conf_value = map.get(scheme_key);
            if let Some(cv) = conf_value {
                if !validate_type(scheme_value, cv) {
                    return Err(format!("[conf/scheme]: Does not match the type specified in the schema {}: {}", idx + 1, line));
                }
            }
        }
    }

    Ok(map)
}

fn validate_type(type_str: &str, value: &str) -> bool {
    match type_str {
        constant::CONF_TYPE_STRING => value.parse::<String>().is_ok(),
        constant::CONF_TYPE_BOOL => value.parse::<bool>().is_ok(),
        constant::CONF_TYPE_INT => value.parse::<i32>().is_ok(),
        constant::CONF_TYPE_UINT => value.parse::<u32>().is_ok(),
        constant::CONF_TYPE_FLOAT => value.parse::<f64>().is_ok(),
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

fn skip_line(str: &str) -> bool {
    str.is_empty() || str.starts_with('#') || str.starts_with(';')
}


