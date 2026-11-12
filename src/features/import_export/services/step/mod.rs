//! Streaming STEP file reader: splits the byte stream into statements and keeps only the entities asked for.

mod value;

pub use value::{parse_value, unescape, Value};

use crate::features::import_export::models::ImportError;
use crate::features::import_export::services::progress::Progress;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;

/// An entity instance: `#id = TYPE(args)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    /// Uppercase type name.
    pub ty: String,
    /// Attributes in order.
    pub args: Vec<Value>,
}

impl Entity {
    /// Attribute `index`, or `Null` when absent.
    pub fn arg(&self, index: usize) -> &Value {
        const NULL: Value = Value::Null;
        self.args.get(index).unwrap_or(&NULL)
    }
}

/// Header section data.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Header {
    /// Schema names from `FILE_SCHEMA`.
    pub schemas: Vec<String>,
    /// `FILE_NAME` preprocessor version (usually names the exporter).
    pub preprocessor: String,
    /// `FILE_NAME` originating system (usually the authoring application).
    pub originating_system: String,
}

/// A parsed STEP file.
#[derive(Debug, Default)]
pub struct StepFile {
    /// Header data.
    pub header: Header,
    /// Entities that passed the filter.
    pub entities: HashMap<u32, Entity>,
    /// Count of entities dropped by the filter, by type.
    pub skipped: BTreeMap<String, usize>,
    /// Entities that could not be parsed.
    pub malformed: usize,
    /// Entities read in total.
    pub total: usize,
}

impl StepFile {
    /// The entity with `id`.
    pub fn get(&self, id: u32) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// The entity referenced by attribute `index` of `entity`.
    pub fn follow(&self, entity: &Entity, index: usize) -> Option<&Entity> {
        entity.arg(index).as_ref().and_then(|id| self.get(id))
    }
}

struct Reader<'a> {
    file: StepFile,
    keep: &'a dyn Fn(&str) -> bool,
    in_header: bool,
}

impl Reader<'_> {
    fn statement(&mut self, bytes: &[u8]) {
        let text = bytes.trim_ascii();
        if text.is_empty() {
            return;
        }
        if text.first() == Some(&b'#') {
            self.entity(text);
            return;
        }
        let name_end = text.iter().position(|b| *b == b'(').unwrap_or(text.len());
        let name = String::from_utf8_lossy(&text[..name_end])
            .trim()
            .to_ascii_uppercase();
        match name.as_str() {
            "HEADER" => self.in_header = true,
            "DATA" | "ENDSEC" => self.in_header = false,
            "FILE_SCHEMA" | "FILE_NAME" if name_end < text.len() => {
                if let Ok(list) = parse_value(&text[name_end..]) {
                    self.header_statement(&name, &list.items());
                }
            }
            _ => {}
        }
    }

    fn header_statement(&mut self, name: &str, args: &[Value]) {
        if name == "FILE_SCHEMA" {
            self.file.header.schemas = args
                .first()
                .map(|list| {
                    list.items()
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
        } else {
            let text = |index: usize| {
                args.get(index)
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned()
            };
            self.file.header.preprocessor = text(4);
            self.file.header.originating_system = text(5);
        }
    }

    fn entity(&mut self, text: &[u8]) {
        self.file.total += 1;
        let Some(equals) = text.iter().position(|b| *b == b'=') else {
            self.file.malformed += 1;
            return;
        };
        let id: Option<u32> = std::str::from_utf8(&text[1..equals])
            .ok()
            .and_then(|t| t.trim().parse().ok());
        let rest = text[equals + 1..].trim_ascii();
        let name_end = rest.iter().position(|b| *b == b'(').unwrap_or(rest.len());
        let (Some(id), true) = (id, name_end < rest.len()) else {
            self.file.malformed += 1;
            return;
        };
        let ty = String::from_utf8_lossy(&rest[..name_end])
            .trim()
            .to_ascii_uppercase();
        if ty.is_empty() {
            *self
                .file
                .skipped
                .entry("(entidad compleja)".into())
                .or_insert(0) += 1;
            return;
        }
        if !(self.keep)(&ty) {
            *self.file.skipped.entry(ty).or_insert(0) += 1;
            return;
        }
        match parse_value(&rest[name_end..]) {
            Ok(list @ (Value::List(_) | Value::Refs(_) | Value::Reals(_))) => {
                let args = list.items().into_owned();
                self.file.entities.insert(id, Entity { ty, args });
            }
            _ => self.file.malformed += 1,
        }
    }
}

/// Reads a STEP stream, keeping entities whose type passes `keep`. `on_progress` receives 0..=1 as bytes are read.
pub fn parse_stream<R: Read>(
    mut reader: R,
    total_bytes: u64,
    keep: &dyn Fn(&str) -> bool,
    progress: &Progress,
    scale: (f32, f32),
) -> Result<StepFile, ImportError> {
    let mut state = Reader {
        file: StepFile::default(),
        keep,
        in_header: false,
    };
    let mut buffer = vec![0u8; 1 << 20];
    let mut statement: Vec<u8> = Vec::with_capacity(4096);
    let (mut in_string, mut in_comment, mut previous) = (false, false, 0u8);
    let mut read_total: u64 = 0;
    loop {
        progress.check()?;
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        for &byte in &buffer[..count] {
            if in_comment {
                if previous == b'*' && byte == b'/' {
                    in_comment = false;
                }
            } else if in_string {
                statement.push(byte);
                if byte == b'\'' {
                    in_string = false;
                }
            } else if previous == b'/' && byte == b'*' {
                statement.pop();
                in_comment = true;
            } else if byte == b'\'' {
                in_string = true;
                statement.push(byte);
            } else if byte == b';' {
                state.statement(&statement);
                statement.clear();
            } else {
                statement.push(byte);
            }
            previous = byte;
        }
        read_total += count as u64;
        let fraction = if total_bytes == 0 {
            0.0
        } else {
            read_total as f32 / total_bytes as f32
        };
        progress.set(scale.0 + (scale.1 - scale.0) * fraction.min(1.0));
    }
    if !statement.trim_ascii().is_empty() {
        state.statement(&statement);
    }
    Ok(state.file)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION(('ViewDefinition [CoordinationView]'),'2;1');\nFILE_NAME('a.ifc','2024-01-01',('A'),('B'),'Autodesk Revit 2024','Autodesk Revit 2024 (ENU)','');\nFILE_SCHEMA(('IFC2X3'));\nENDSEC;\nDATA;\n#1=IFCPROJECT('0',#2,'Proyecto con ; punto y coma',$,$,$,$,(#3),#4);\n#2=IFCPROPERTYSINGLEVALUE('x',$,IFCTEXT('y'),$);\n/* comment; */\n#3=IFCCARTESIANPOINT((0.,\n 1.,2.));\nENDSEC;\nEND-ISO-10303-21;\n";

    #[test]
    fn reads_header_entities_and_skips_filtered_types() {
        let progress = Progress::default();
        let keep = |ty: &str| !ty.starts_with("IFCPROPERTY");
        let file = parse_stream(
            SAMPLE.as_bytes(),
            SAMPLE.len() as u64,
            &keep,
            &progress,
            (0.0, 1.0),
        )
        .unwrap();
        assert_eq!(file.header.schemas, vec!["IFC2X3"]);
        assert_eq!(file.header.originating_system, "Autodesk Revit 2024 (ENU)");
        assert_eq!(file.entities.len(), 2);
        assert_eq!(file.skipped.get("IFCPROPERTYSINGLEVALUE"), Some(&1));
        assert_eq!(
            file.get(1).unwrap().arg(2).as_str(),
            Some("Proyecto con ; punto y coma")
        );
        assert_eq!(file.get(3).unwrap().arg(0).items().len(), 3);
        assert_eq!(file.malformed, 0);
    }

    #[test]
    fn cancellation_stops_the_parse() {
        let progress = Progress::default();
        progress.cancel();
        let result = parse_stream(SAMPLE.as_bytes(), 10, &|_| true, &progress, (0.0, 1.0));
        assert_eq!(result.err(), Some(ImportError::Cancelled));
    }
}
