use std::env;
use std::fs::read_to_string;
use toml::value::Value;
use toml::Table;
use std::path::Path;

pub fn get_env_var(key: &str) -> String {
    for (k, v) in env::vars() {
        if k == key {
            return v;
        }
    }

    panic!("Need env var: {}", key);
}

fn get_toml_string_rec(table: &Table, mut path: Vec<&str>) -> Option<String> {
    if path.len() == 1 {
        if !table.contains_key(path[0]) {
            return None;
        }

        return match table[path[0]].clone() {
            Value::String(s) => Some(s),
            _ => None,
        };
    }

    let id = path.remove(0);
	if !table.contains_key(id) {
		return None
	}

    match table[id].clone() {
        Value::Table(t) => get_toml_string_rec(&t, path),
        _ => None,
    }
}

pub fn get_toml_string(file_name: &Path, path: Vec<&str>) -> Option<String> {
    let config = {
        let f = read_to_string(file_name);
        if let Ok(f) = f {
            f.parse::<Table>().unwrap()
        } else {
            panic!("Unable to read {}",file_name.display());
        }
    };

    get_toml_string_rec(&config, path)
}
