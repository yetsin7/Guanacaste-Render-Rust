//! Minimal standard-alphabet base64 decoder for the binary arrays Guanacaste 3D stores inside its JSON.
//! Tolerant of padding and whitespace; returns `None` on any other byte.

const INVALID: u8 = 0xFF;

fn table() -> &'static [u8; 256] {
    static TABLE: std::sync::OnceLock<[u8; 256]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [INVALID; 256];
        for (value, symbol) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
            .iter()
            .enumerate()
        {
            table[*symbol as usize] = value as u8;
        }
        table
    })
}

/// Decodes `text`; `None` when it contains a byte outside the base64 alphabet.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let table = table();
    let mut out = Vec::with_capacity(text.len() / 4 * 3 + 3);
    let (mut accumulator, mut bits) = (0u32, 0u32);
    for &byte in text.as_bytes() {
        match byte {
            b'=' | b'\r' | b'\n' | b' ' => continue,
            _ => {}
        }
        let value = table[byte as usize];
        if value == INVALID {
            return None;
        }
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((accumulator >> bits) as u8);
            accumulator &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// Decodes little-endian `f32` values.
pub fn floats(text: &str) -> Option<Vec<f32>> {
    let bytes = decode(text)?;
    Some(
        bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}

/// Decodes little-endian `u32` values.
pub fn words(text: &str) -> Option<Vec<u32>> {
    let bytes = decode(text)?;
    Some(
        bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_with_and_without_padding() {
        assert_eq!(decode("TWFu").unwrap(), b"Man");
        assert_eq!(decode("TWE=").unwrap(), b"Ma");
        assert_eq!(decode("TQ==").unwrap(), b"M");
        assert_eq!(decode("TQ").unwrap(), b"M");
        assert!(decode("T!Fu").is_none());
    }

    #[test]
    fn decodes_floats_little_endian() {
        // 1.0f32 = 00 00 80 3F
        assert_eq!(floats("AACAPw==").unwrap(), vec![1.0]);
    }
}
