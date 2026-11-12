//! Binary FBX container reader: node records, properties and zlib-compressed arrays.

use crate::features::import_export::models::ImportError;

const MAGIC: &[u8] = b"Kaydara FBX Binary  \0";

#[derive(Debug, Clone)]
pub(super) enum Prop {
    Int(i64),
    Float(f64),
    Text(String),
    Floats(Vec<f64>),
    Ints(Vec<i64>),
}

#[derive(Debug, Clone, Default)]
pub(super) struct Node {
    pub(super) name: String,
    pub(super) props: Vec<Prop>,
    pub(super) children: Vec<Node>,
}

impl Node {
    pub(super) fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }
    pub(super) fn int(&self, index: usize) -> Option<i64> {
        match self.props.get(index)? {
            Prop::Int(v) => Some(*v),
            _ => None,
        }
    }
    pub(super) fn text(&self, index: usize) -> &str {
        match self.props.get(index) {
            Some(Prop::Text(t)) => t,
            _ => "",
        }
    }
    pub(super) fn number(&self, index: usize) -> Option<f64> {
        match self.props.get(index)? {
            Prop::Float(v) => Some(*v),
            Prop::Int(v) => Some(*v as f64),
            _ => None,
        }
    }
}

struct Bytes<'a> {
    data: &'a [u8],
    at: usize,
}

impl Bytes<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], ImportError> {
        let slice = self
            .data
            .get(self.at..self.at + n)
            .ok_or_else(|| ImportError::Invalid("FBX truncado".into()))?;
        self.at += n;
        Ok(slice)
    }
    fn u32(&mut self) -> Result<u32, ImportError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, ImportError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
}

fn array(bytes: &mut Bytes, kind: u8) -> Result<Prop, ImportError> {
    let (count, encoding, compressed) =
        (bytes.u32()? as usize, bytes.u32()?, bytes.u32()? as usize);
    let size = match kind {
        b'd' | b'l' => 8,
        b'f' | b'i' => 4,
        _ => 1,
    };
    let raw = bytes.take(compressed)?.to_vec();
    let data = if encoding == 1 {
        miniz_oxide::inflate::decompress_to_vec_zlib(&raw)
            .map_err(|_| ImportError::Invalid("Datos FBX comprimidos no válidos".into()))?
    } else {
        raw
    };
    if data.len() < count * size {
        return Err(ImportError::Invalid("Matriz FBX incompleta".into()));
    }
    let chunks = data.chunks_exact(size).take(count);
    Ok(match kind {
        b'd' => Prop::Floats(
            chunks
                .map(|c| f64::from_le_bytes(c.try_into().unwrap_or([0; 8])))
                .collect(),
        ),
        b'f' => Prop::Floats(
            chunks
                .map(|c| f32::from_le_bytes(c.try_into().unwrap_or([0; 4])) as f64)
                .collect(),
        ),
        b'l' => Prop::Ints(
            chunks
                .map(|c| i64::from_le_bytes(c.try_into().unwrap_or([0; 8])))
                .collect(),
        ),
        b'i' => Prop::Ints(
            chunks
                .map(|c| i32::from_le_bytes(c.try_into().unwrap_or([0; 4])) as i64)
                .collect(),
        ),
        _ => Prop::Ints(chunks.map(|c| c[0] as i64).collect()),
    })
}

fn property(bytes: &mut Bytes) -> Result<Prop, ImportError> {
    let kind = bytes.take(1)?[0];
    Ok(match kind {
        b'Y' => {
            let b = bytes.take(2)?;
            Prop::Int(i16::from_le_bytes([b[0], b[1]]) as i64)
        }
        b'C' => Prop::Int(bytes.take(1)?[0] as i64),
        b'I' => Prop::Int(bytes.u32()? as i32 as i64),
        b'L' => Prop::Int(bytes.u64()? as i64),
        b'F' => Prop::Float(f32::from_bits(bytes.u32()?) as f64),
        b'D' => Prop::Float(f64::from_bits(bytes.u64()?)),
        b'S' | b'R' => {
            let length = bytes.u32()? as usize;
            Prop::Text(String::from_utf8_lossy(bytes.take(length)?).into_owned())
        }
        b'f' | b'd' | b'l' | b'i' | b'b' => array(bytes, kind)?,
        other => {
            return Err(ImportError::Invalid(format!(
                "Tipo de propiedad FBX desconocido: {other}"
            )))
        }
    })
}

fn record(bytes: &mut Bytes, wide: bool, depth: usize) -> Result<Option<Node>, ImportError> {
    if depth > 64 {
        return Err(ImportError::Invalid("FBX demasiado anidado".into()));
    }
    let (end, count, _list_len) = if wide {
        (bytes.u64()? as usize, bytes.u64()? as usize, bytes.u64()?)
    } else {
        (
            bytes.u32()? as usize,
            bytes.u32()? as usize,
            bytes.u32()? as u64,
        )
    };
    let name_length = bytes.take(1)?[0] as usize;
    if end == 0 {
        return Ok(None);
    }
    let name = String::from_utf8_lossy(bytes.take(name_length)?).into_owned();
    let mut node = Node {
        name,
        ..Default::default()
    };
    for _ in 0..count {
        node.props.push(property(bytes)?);
    }
    while bytes.at < end {
        match record(bytes, wide, depth + 1)? {
            Some(child) => node.children.push(child),
            None => break,
        }
    }
    bytes.at = end;
    Ok(Some(node))
}

pub(super) fn parse(data: &[u8]) -> Result<(u32, Vec<Node>), ImportError> {
    if !data.starts_with(MAGIC) {
        return Err(ImportError::Invalid(
            "Este FBX está en formato ASCII o no es un FBX: expórtalo como FBX binario.".into(),
        ));
    }
    let mut bytes = Bytes {
        data,
        at: MAGIC.len() + 2,
    };
    let version = bytes.u32()?;
    let mut nodes = Vec::new();
    while bytes.at + 13 < data.len() {
        match record(&mut bytes, version >= 7500, 0)? {
            Some(node) => nodes.push(node),
            None => break,
        }
    }
    Ok((version, nodes))
}
