//! Tolerant accessors for the JSON document of a Guanacaste 3D project: a missing or mistyped key reads as zero or
//! empty, because the file is untrusted input.

use serde_json::Value;

/// A number, or 0 when absent or not numeric.
pub fn number(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

/// An integer (enumerations are stored as integers), or 0.
pub fn int(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or(0)
}

/// A flag, or `default`.
pub fn flag(value: &Value, key: &str, default: bool) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(default)
}

/// A plan point stored as `{"X":..,"Y":..}`, or the origin.
pub fn point(value: &Value, key: &str) -> [f64; 2] {
    value
        .get(key)
        .map_or([0.0, 0.0], |p| [number(p, "X"), number(p, "Y")])
}

/// A string, or the empty string.
pub fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

/// An array, or the empty slice.
pub fn list<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// A list of plan points stored as `[{"X":..,"Y":..}, ...]`.
pub fn points(value: &Value, key: &str) -> Vec<[f64; 2]> {
    list(value, key)
        .iter()
        .map(|p| [number(p, "X"), number(p, "Y")])
        .collect()
}
