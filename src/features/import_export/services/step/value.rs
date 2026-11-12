//! STEP (ISO 10303-21) attribute values and their parser.

/// An attribute value of a STEP entity.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `$`.
    Null,
    /// `*`.
    Omitted,
    /// Integer.
    Int(i64),
    /// Real number.
    Real(f64),
    /// String, already unescaped.
    Str(String),
    /// Enumeration or boolean, without the dots, uppercase.
    Enum(String),
    /// Reference `#123`.
    Ref(u32),
    /// List.
    List(Vec<Value>),
    /// Typed value such as `IFCLENGTHMEASURE(1.5)`.
    Typed(String, Vec<Value>),
}

impl Value {
    /// The referenced entity id.
    pub fn as_ref(&self) -> Option<u32> {
        match self {
            Value::Ref(id) => Some(*id),
            _ => None,
        }
    }

    /// A number from an integer, real or typed measure.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(value) => Some(*value as f64),
            Value::Real(value) => Some(*value),
            Value::Typed(_, inner) => inner.first().and_then(Value::as_f64),
            _ => None,
        }
    }

    /// An integer.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(value) => Some(*value),
            Value::Real(value) => Some(*value as i64),
            _ => None,
        }
    }

    /// A string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(text) => Some(text),
            _ => None,
        }
    }

    /// An enumeration literal.
    pub fn as_enum(&self) -> Option<&str> {
        match self {
            Value::Enum(text) => Some(text),
            _ => None,
        }
    }

    /// The items of a list (empty for anything else).
    pub fn items(&self) -> &[Value] {
        match self {
            Value::List(items) => items,
            _ => &[],
        }
    }

    /// Ids of every reference in a list (or the single reference).
    pub fn refs(&self) -> Vec<u32> {
        match self {
            Value::Ref(id) => vec![*id],
            Value::List(items) => items.iter().filter_map(Value::as_ref).collect(),
            _ => Vec::new(),
        }
    }
}

/// Decodes STEP string escapes: `''`, `\X2\...\X0\`, `\X\hh` and `\S\c`.
pub fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let chars: Vec<char> = raw.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\\' {
            let rest: String = chars[index..chars.len().min(index + 4)].iter().collect();
            if rest.starts_with("\\X2\\") {
                let mut end = index + 4;
                let mut hex = String::new();
                while end < chars.len()
                    && !(chars[end] == '\\'
                        && chars[end..].iter().take(4).collect::<String>() == "\\X0\\")
                {
                    hex.push(chars[end]);
                    end += 1;
                }
                let units: Vec<u16> = hex
                    .as_bytes()
                    .chunks(4)
                    .filter_map(|chunk| {
                        u16::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()
                    })
                    .collect();
                out.push_str(&String::from_utf16_lossy(&units));
                index = (end + 4).min(chars.len());
                continue;
            }
            if rest.starts_with("\\X\\") && index + 5 <= chars.len() {
                let hex: String = chars[index + 3..index + 5].iter().collect();
                if let Ok(code) = u8::from_str_radix(&hex, 16) {
                    out.push(code as char);
                    index += 5;
                    continue;
                }
            }
            if rest.starts_with("\\S\\") && index + 4 <= chars.len() {
                out.push(((chars[index + 3] as u32 + 128) as u8) as char);
                index += 4;
                continue;
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Cursor<'_> {
    fn skip_space(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
}

fn parse_inner(cursor: &mut Cursor, depth: usize) -> Result<Value, String> {
    if depth > 64 {
        return Err("nesting too deep".into());
    }
    cursor.skip_space();
    let Some(byte) = cursor.peek() else {
        return Err("unexpected end".into());
    };
    match byte {
        b'$' => {
            cursor.pos += 1;
            Ok(Value::Null)
        }
        b'*' => {
            cursor.pos += 1;
            Ok(Value::Omitted)
        }
        b'#' => {
            cursor.pos += 1;
            let start = cursor.pos;
            while cursor.peek().is_some_and(|b| b.is_ascii_digit()) {
                cursor.pos += 1;
            }
            std::str::from_utf8(&cursor.bytes[start..cursor.pos])
                .ok()
                .and_then(|t| t.parse().ok())
                .map(Value::Ref)
                .ok_or_else(|| "bad reference".to_owned())
        }
        b'\'' => {
            cursor.pos += 1;
            let mut raw = Vec::new();
            loop {
                match cursor.peek() {
                    None => return Err("unterminated string".into()),
                    Some(b'\'') => {
                        if cursor.bytes.get(cursor.pos + 1) == Some(&b'\'') {
                            raw.push(b'\'');
                            cursor.pos += 2;
                        } else {
                            cursor.pos += 1;
                            break;
                        }
                    }
                    Some(other) => {
                        raw.push(other);
                        cursor.pos += 1;
                    }
                }
            }
            Ok(Value::Str(unescape(&String::from_utf8_lossy(&raw))))
        }
        b'.' => {
            cursor.pos += 1;
            let start = cursor.pos;
            while cursor.peek().is_some_and(|b| b != b'.') {
                cursor.pos += 1;
            }
            let text =
                String::from_utf8_lossy(&cursor.bytes[start..cursor.pos]).to_ascii_uppercase();
            cursor.pos += 1;
            Ok(Value::Enum(text))
        }
        b'(' => {
            cursor.pos += 1;
            parse_list(cursor, depth).map(Value::List)
        }
        b'+' | b'-' | b'0'..=b'9' => {
            let start = cursor.pos;
            let mut real = false;
            while let Some(b) = cursor.peek() {
                match b {
                    b'0'..=b'9' | b'+' | b'-' => {}
                    b'.' | b'E' | b'e' => real = true,
                    _ => break,
                }
                cursor.pos += 1;
            }
            let text =
                std::str::from_utf8(&cursor.bytes[start..cursor.pos]).map_err(|_| "bad number")?;
            if real {
                text.parse()
                    .map(Value::Real)
                    .map_err(|_| format!("bad number {text}"))
            } else {
                text.parse()
                    .map(Value::Int)
                    .map_err(|_| format!("bad number {text}"))
            }
        }
        b if b.is_ascii_alphabetic() || b == b'_' => {
            let start = cursor.pos;
            while cursor
                .peek()
                .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                cursor.pos += 1;
            }
            let name =
                String::from_utf8_lossy(&cursor.bytes[start..cursor.pos]).to_ascii_uppercase();
            cursor.skip_space();
            if cursor.peek() == Some(b'(') {
                cursor.pos += 1;
                parse_list(cursor, depth).map(|args| Value::Typed(name, args))
            } else {
                Ok(Value::Enum(name))
            }
        }
        other => Err(format!("unexpected character {}", other as char)),
    }
}

fn parse_list(cursor: &mut Cursor, depth: usize) -> Result<Vec<Value>, String> {
    let mut items = Vec::new();
    loop {
        cursor.skip_space();
        match cursor.peek() {
            Some(b')') => {
                cursor.pos += 1;
                return Ok(items);
            }
            Some(b',') => cursor.pos += 1,
            None => return Err("unterminated list".into()),
            _ => items.push(parse_inner(cursor, depth + 1)?),
        }
    }
}

/// Parses one value starting at `bytes[0]`; trailing bytes are ignored.
pub fn parse_value(bytes: &[u8]) -> Result<Value, String> {
    parse_inner(&mut Cursor { bytes, pos: 0 }, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_values() {
        let value =
            parse_value(b"(#12,$,*,'a''b',.T.,1.5E-3,-4,IFCLENGTHMEASURE(2.),(1,2))").unwrap();
        let items = value.items();
        assert_eq!(items[0], Value::Ref(12));
        assert_eq!(items[1], Value::Null);
        assert_eq!(items[3], Value::Str("a'b".into()));
        assert_eq!(items[4], Value::Enum("T".into()));
        assert_eq!(items[5].as_f64(), Some(0.0015));
        assert_eq!(items[6], Value::Int(-4));
        assert_eq!(items[7].as_f64(), Some(2.0));
        assert_eq!(items[8].items().len(), 2);
    }

    #[test]
    fn unescapes_unicode_sequences() {
        assert_eq!(unescape("Muro b\\X2\\00E1\\X0\\sico"), "Muro básico");
        assert_eq!(unescape("Dise\\X\\F1o"), "Diseño");
    }

    #[test]
    fn malformed_input_is_an_error_not_a_panic() {
        assert!(parse_value(b"(1,2").is_err());
        assert!(parse_value(b"'abc").is_err());
        assert!(parse_value(b"@").is_err());
    }
}
