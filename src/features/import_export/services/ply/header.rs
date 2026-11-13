//! PLY header and value reader (ASCII, binary little and big endian).

use crate::features::import_export::models::ImportError;

/// A scalar type of the PLY format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scalar {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
}

impl Scalar {
    fn parse(word: &str) -> Option<Scalar> {
        Some(match word {
            "char" | "int8" => Scalar::I8,
            "uchar" | "uint8" => Scalar::U8,
            "short" | "int16" => Scalar::I16,
            "ushort" | "uint16" => Scalar::U16,
            "int" | "int32" => Scalar::I32,
            "uint" | "uint32" => Scalar::U32,
            "float" | "float32" => Scalar::F32,
            "double" | "float64" => Scalar::F64,
            _ => return None,
        })
    }

    /// Size in the binary encodings.
    pub fn size(self) -> usize {
        match self {
            Scalar::I8 | Scalar::U8 => 1,
            Scalar::I16 | Scalar::U16 => 2,
            Scalar::I32 | Scalar::U32 | Scalar::F32 => 4,
            Scalar::F64 => 8,
        }
    }
}

/// One property of an element: a scalar, or a list with a count type.
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub kind: Scalar,
    pub count: Option<Scalar>,
}

/// An element type (`vertex`, `face`...) with its instance count.
#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub count: usize,
    pub props: Vec<Property>,
}

/// How the body is encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Ascii,
    Little,
    Big,
}

/// The parsed header.
#[derive(Debug)]
pub struct Header {
    pub encoding: Encoding,
    pub elements: Vec<Element>,
    pub texture: Option<String>,
    pub body: usize,
}

fn bad(text: &str) -> ImportError {
    ImportError::Invalid(format!("Cabecera PLY no válida: {text}."))
}

/// Parses the header of `bytes`.
pub fn parse_header(bytes: &[u8]) -> Result<Header, ImportError> {
    if !bytes.starts_with(b"ply") {
        return Err(bad("falta la firma «ply»"));
    }
    let end = bytes
        .windows(10)
        .position(|w| w == b"end_header")
        .ok_or_else(|| bad("falta end_header"))?;
    let mut body = end + 10;
    while body < bytes.len() && bytes[body] != b'\n' {
        body += 1;
    }
    body = (body + 1).min(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]);
    let mut header = Header {
        encoding: Encoding::Ascii,
        elements: Vec::new(),
        texture: None,
        body,
    };
    let mut format_seen = false;
    for line in text.lines().skip(1) {
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["format", kind, ..] => {
                format_seen = true;
                header.encoding = match *kind {
                    "ascii" => Encoding::Ascii,
                    "binary_little_endian" => Encoding::Little,
                    "binary_big_endian" => Encoding::Big,
                    _ => return Err(bad("codificación desconocida")),
                };
            }
            ["comment", "TextureFile", name, ..] => header.texture = Some((*name).to_owned()),
            ["element", name, count] => header.elements.push(Element {
                name: (*name).to_owned(),
                count: count.parse().map_err(|_| bad("recuento de elementos"))?,
                props: Vec::new(),
            }),
            ["property", "list", count, kind, name] => {
                let property = Property {
                    name: (*name).to_owned(),
                    kind: Scalar::parse(kind).ok_or_else(|| bad("tipo de propiedad"))?,
                    count: Some(Scalar::parse(count).ok_or_else(|| bad("tipo de recuento"))?),
                };
                header
                    .elements
                    .last_mut()
                    .ok_or_else(|| bad("propiedad sin elemento"))?
                    .props
                    .push(property);
            }
            ["property", kind, name] => {
                let property = Property {
                    name: (*name).to_owned(),
                    kind: Scalar::parse(kind).ok_or_else(|| bad("tipo de propiedad"))?,
                    count: None,
                };
                header
                    .elements
                    .last_mut()
                    .ok_or_else(|| bad("propiedad sin elemento"))?
                    .props
                    .push(property);
            }
            _ => {}
        }
    }
    if !format_seen {
        return Err(bad("falta la línea format"));
    }
    Ok(header)
}

/// Sequential value reader over the body.
pub enum Reader<'a> {
    /// ASCII tokens.
    Ascii(std::str::SplitAsciiWhitespace<'a>),
    /// Binary bytes with a cursor and byte order.
    Binary {
        bytes: &'a [u8],
        at: usize,
        big: bool,
    },
}

impl Reader<'_> {
    /// Next value as `f64`, or `None` at the end of the data.
    pub fn read(&mut self, kind: Scalar) -> Option<f64> {
        match self {
            Reader::Ascii(tokens) => tokens.next()?.parse().ok(),
            Reader::Binary { bytes, at, big } => {
                let size = kind.size();
                let mut raw = [0u8; 8];
                raw[..size].copy_from_slice(bytes.get(*at..*at + size)?);
                *at += size;
                if *big {
                    raw[..size].reverse();
                }
                Some(match kind {
                    Scalar::I8 => raw[0] as i8 as f64,
                    Scalar::U8 => raw[0] as f64,
                    Scalar::I16 => i16::from_le_bytes([raw[0], raw[1]]) as f64,
                    Scalar::U16 => u16::from_le_bytes([raw[0], raw[1]]) as f64,
                    Scalar::I32 => i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as f64,
                    Scalar::U32 => u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as f64,
                    Scalar::F32 => f32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as f64,
                    Scalar::F64 => f64::from_le_bytes(raw),
                })
            }
        }
    }

    /// Bytes (binary) or characters (ASCII) not read yet, an upper bound for plausibility checks.
    pub fn remaining(&self) -> usize {
        match self {
            Reader::Ascii(tokens) => tokens.clone().map(|t| t.len() + 1).sum(),
            Reader::Binary { bytes, at, .. } => bytes.len().saturating_sub(*at),
        }
    }
}
