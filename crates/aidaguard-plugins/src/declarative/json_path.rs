use serde_json::Value;

/// Read a value from a JSON object using dot-notation path.
/// Supports: `key`, `a.b.c`, `a.*.c` (wildcard on objects/arrays), `a.0.c` (array index).
pub fn json_get<'a>(root: &'a Value, path: &str) -> Option<&'a str> {
    let segments: Vec<&str> = path.split('.').collect();
    json_get_segments(root, &segments)
}

fn json_get_segments<'a>(node: &'a Value, segments: &[&str]) -> Option<&'a str> {
    if segments.is_empty() {
        return node.as_str();
    }
    let head = segments[0];
    let tail = &segments[1..];

    if head == "*" {
        // Wildcard: search in array elements or object values
        match node {
            Value::Array(arr) => {
                for item in arr {
                    if let Some(v) = json_get_segments(item, tail) {
                        return Some(v);
                    }
                }
                None
            }
            Value::Object(map) => {
                for (_, val) in map {
                    if let Some(v) = json_get_segments(val, tail) {
                        return Some(v);
                    }
                }
                None
            }
            _ => None,
        }
    } else if let Ok(idx) = head.parse::<usize>() {
        // Numeric index
        node.get(idx).and_then(|v| json_get_segments(v, tail))
    } else {
        // Named key
        node.get(head).and_then(|v| json_get_segments(v, tail))
    }
}

/// Write a value into a JSON object at the given dot-notation path.
/// Creates intermediate objects as needed. Returns the new value.
/// Passing `None` as value removes the key at the final segment.
pub fn json_set(root: &mut Value, path: &str, value: Option<&str>) {
    let segments: Vec<&str> = path.split('.').collect();
    json_set_segments(root, &segments, value);
}

fn json_set_segments(node: &mut Value, segments: &[&str], value: Option<&str>) {
    if segments.is_empty() {
        if let Some(s) = value {
            *node = Value::String(s.to_string());
        }
        return;
    }

    let head = segments[0];
    let tail = &segments[1..];

    if head == "*" {
        // Wildcard: apply to all array elements or object values
        match node {
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    json_set_segments(item, tail, value);
                }
            }
            Value::Object(map) => {
                for (_, val) in map.iter_mut() {
                    json_set_segments(val, tail, value);
                }
            }
            _ => {}
        }
    } else if let Ok(idx) = head.parse::<usize>() {
        // Ensure array is large enough
        if !node.is_array() {
            *node = Value::Array(Vec::new());
        }
        if let Value::Array(arr) = node {
            while arr.len() <= idx {
                arr.push(if tail.is_empty() { Value::Null } else { Value::Object(serde_json::Map::new()) });
            }
            if tail.is_empty() {
                if let Some(s) = value {
                    arr[idx] = Value::String(s.to_string());
                } else if idx < arr.len() {
                    arr.remove(idx);
                }
            } else {
                json_set_segments(&mut arr[idx], tail, value);
            }
        }
    } else {
        // Named key: ensure the object exists
        if !node.is_object() {
            *node = Value::Object(serde_json::Map::new());
        }
        if let Value::Object(map) = node {
            if tail.is_empty() {
                if let Some(s) = value {
                    map.insert(head.to_string(), Value::String(s.to_string()));
                } else {
                    map.remove(head);
                }
            } else {
                let child = map
                    .entry(head.to_string())
                    .or_insert(Value::Object(serde_json::Map::new()));
                json_set_segments(child, tail, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_key() {
        let json: Value = serde_json::json!({"key": "value"});
        assert_eq!(json_get(&json, "key"), Some("value"));
    }

    #[test]
    fn test_nested_key() {
        let json: Value = serde_json::json!({"a": {"b": {"c": "deep"}}});
        assert_eq!(json_get(&json, "a.b.c"), Some("deep"));
    }

    #[test]
    fn test_missing_key() {
        let json: Value = serde_json::json!({"a": 1});
        assert_eq!(json_get(&json, "b"), None);
    }

    #[test]
    fn test_array_index() {
        let json: Value = serde_json::json!({"arr": ["zero", "one", "two"]});
        assert_eq!(json_get(&json, "arr.1"), Some("one"));
    }

    #[test]
    fn test_wildcard_object() {
        let json: Value = serde_json::json!({
            "providers": {
                "a": {"url": "http://a.com"},
                "b": {"url": "http://b.com"}
            }
        });
        // Wildcard returns first match
        let result = json_get(&json, "providers.*.url");
        assert!(result == Some("http://a.com") || result == Some("http://b.com"));
    }

    #[test]
    fn test_json_set_nested() {
        let mut json: Value = serde_json::json!({});
        json_set(&mut json, "env.ANTHROPIC_BASE_URL", Some("http://proxy:19000"));
        assert_eq!(json_get(&json, "env.ANTHROPIC_BASE_URL"), Some("http://proxy:19000"));
    }

    #[test]
    fn test_json_set_delete() {
        let mut json: Value = serde_json::json!({"key": "value"});
        json_set(&mut json, "key", None);
        assert_eq!(json_get(&json, "key"), None);
    }

    #[test]
    fn test_json_set_wildcard() {
        let mut json: Value = serde_json::json!({
            "models": [
                {"name": "a"},
                {"name": "b"}
            ]
        });
        json_set(&mut json, "models.*.apiBase", Some("http://proxy:19000"));
        assert_eq!(json_get(&json, "models.0.apiBase"), Some("http://proxy:19000"));
        assert_eq!(json_get(&json, "models.1.apiBase"), Some("http://proxy:19000"));
    }
}
