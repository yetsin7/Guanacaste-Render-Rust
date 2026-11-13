//! The `GRRPROJ1` container: round trip, checksums, truncation and load limits.

use super::temp_folder;
use crate::features::projects::services::container::{
    self, BlockSource, ContainerError, Limits, Reader, KIND_MESH,
};
use std::borrow::Cow;
use std::path::Path;

fn write_sample(path: &Path) {
    let mut file = std::fs::File::create(path).expect("create");
    let mut blocks = [BlockSource {
        kind: KIND_MESH,
        id: 0,
        raw: Cow::Owned(vec![7u8; 5000]),
    }]
    .into_iter();
    container::write(
        &mut file,
        1,
        [3u8; 16],
        br#"{"meta":{"name":"Casa"}}"#,
        &mut blocks,
    )
    .expect("write");
}

fn flip(path: &Path, at: usize) {
    let mut bytes = std::fs::read(path).expect("read");
    bytes[at] ^= 0xFF;
    std::fs::write(path, bytes).expect("write");
}

#[test]
fn a_written_container_reads_back_identically() {
    let path = temp_folder("container").join("a.gr");
    write_sample(&path);
    let mut reader = Reader::open(&path, Limits::default()).expect("open");
    assert_eq!(&std::fs::read(&path).unwrap()[..8], b"GRRPROJ1");
    assert_eq!(reader.header.document_id, [3u8; 16]);
    assert_eq!(reader.json, br#"{"meta":{"name":"Casa"}}"#);
    assert_eq!(reader.read_block(0).expect("block"), vec![7u8; 5000]);
}

#[test]
fn damage_anywhere_is_detected() {
    let path = temp_folder("damage").join("a.gr");
    write_sample(&path);
    let length = std::fs::metadata(&path).unwrap().len() as usize;
    flip(&path, 60);
    assert!(matches!(
        Reader::open(&path, Limits::default()),
        Err(ContainerError::Damaged(_))
    ));
    write_sample(&path);
    let block_offset = Reader::open(&path, Limits::default()).unwrap().blocks[0].offset as usize;
    flip(&path, block_offset + 12);
    let mut reader = Reader::open(&path, Limits::default()).expect("header and JSON are intact");
    assert!(matches!(
        reader.read_block(0),
        Err(ContainerError::Damaged(_))
    ));
    write_sample(&path);
    let bytes = std::fs::read(&path).unwrap();
    std::fs::write(&path, &bytes[..length - 10]).unwrap();
    assert!(matches!(
        Reader::open(&path, Limits::default()),
        Err(ContainerError::Damaged(_))
    ));
}

#[test]
fn other_files_are_not_projects() {
    let path = temp_folder("other").join("a.gr");
    std::fs::write(&path, b"esto no es un proyecto, solo texto que ocupa mas de noventa y seis bytes .......................").unwrap();
    assert_eq!(
        Reader::open(&path, Limits::default()).err(),
        Some(ContainerError::NotAProject)
    );
}

#[test]
fn limits_stop_oversized_content_before_reading_it() {
    let path = temp_folder("limits").join("a.gr");
    write_sample(&path);
    let tiny_block = Limits {
        max_block: 100,
        ..Limits::default()
    };
    assert!(matches!(
        Reader::open(&path, tiny_block),
        Err(ContainerError::TooLarge(_))
    ));
    let tiny_json = Limits {
        max_json: 4,
        ..Limits::default()
    };
    assert!(matches!(
        Reader::open(&path, tiny_json),
        Err(ContainerError::TooLarge(_))
    ));
}
