//! Where a material's texture maps come from: an image file or a procedural generator.

use serde::{Deserialize, Serialize};

/// The procedural patterns that can be generated on the CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProceduralKind {
    /// Running-bond brick wall with mortar joints.
    Brick,
    /// Wooden planks with grain.
    Wood,
    /// Poured concrete with pores and stains.
    Concrete,
    /// Veined marble.
    Marble,
    /// Short grass.
    Grass,
    /// Plain fractal noise.
    Noise,
}

impl ProceduralKind {
    /// Every pattern, in menu order.
    pub const ALL: [ProceduralKind; 6] = [
        ProceduralKind::Brick,
        ProceduralKind::Wood,
        ProceduralKind::Concrete,
        ProceduralKind::Marble,
        ProceduralKind::Grass,
        ProceduralKind::Noise,
    ];

    /// Localization key of the pattern's name.
    pub fn label_key(self) -> &'static str {
        match self {
            ProceduralKind::Brick => "material.proc_brick",
            ProceduralKind::Wood => "material.proc_wood",
            ProceduralKind::Concrete => "material.proc_concrete",
            ProceduralKind::Marble => "material.proc_marble",
            ProceduralKind::Grass => "material.proc_grass",
            ProceduralKind::Noise => "material.proc_noise",
        }
    }
}

/// A procedural pattern with its parameters. The pattern tiles seamlessly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Procedural {
    /// Which pattern.
    pub kind: ProceduralKind,
    /// Feature density: 1.0 is the pattern's natural size per texture tile.
    pub scale: f32,
    /// Random seed; each seed gives a different variation.
    pub seed: u32,
}

impl Procedural {
    /// A pattern of `kind` at natural scale, seed 0.
    pub fn new(kind: ProceduralKind) -> Self {
        Self {
            kind,
            scale: 1.0,
            seed: 0,
        }
    }
}

impl Default for Procedural {
    fn default() -> Self {
        Self::new(ProceduralKind::Noise)
    }
}

/// The origin of a texture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextureSource {
    /// A PNG or JPEG file; the path is stored as given (absolute or relative to the working directory).
    File {
        /// File path.
        path: String,
    },
    /// A generated pattern.
    Procedural(Procedural),
}

impl TextureSource {
    /// A short name for lists: the file name, or the pattern's localization key is used by the caller.
    pub fn file_name(&self) -> Option<&str> {
        match self {
            TextureSource::File { path } => Some(
                path.rsplit(['/', '\\'])
                    .next()
                    .filter(|name| !name.is_empty())
                    .unwrap_or(path),
            ),
            TextureSource::Procedural(_) => None,
        }
    }

    /// A stable text identifying the source and its parameters, used as a cache key.
    pub fn cache_key(&self) -> String {
        match self {
            TextureSource::File { path } => format!("file:{path}"),
            TextureSource::Procedural(p) => {
                format!("proc:{:?}:{}:{}", p.kind, p.scale.to_bits(), p.seed)
            }
        }
    }
}

/// A texture assigned to one map slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureRef {
    /// Where the image comes from.
    pub source: TextureSource,
    /// For the normal slot: the image is a grayscale height (bump) map to convert instead of a normal map.
    #[serde(default)]
    pub is_height: bool,
}

impl TextureRef {
    /// A texture from the file at `path`.
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            source: TextureSource::File { path: path.into() },
            is_height: false,
        }
    }

    /// A procedural texture.
    pub fn procedural(procedural: Procedural) -> Self {
        Self {
            source: TextureSource::Procedural(procedural),
            is_height: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_are_extracted_from_both_separators() {
        let unix = TextureSource::File {
            path: "a/b/brick.png".into(),
        };
        let windows = TextureSource::File {
            path: r"C:\tex\wood.jpg".into(),
        };
        assert_eq!(unix.file_name(), Some("brick.png"));
        assert_eq!(windows.file_name(), Some("wood.jpg"));
        assert_eq!(
            TextureSource::Procedural(Procedural::default()).file_name(),
            None
        );
    }

    #[test]
    fn cache_keys_differ_with_parameters() {
        let a = TextureSource::Procedural(Procedural::new(ProceduralKind::Brick));
        let mut other = Procedural::new(ProceduralKind::Brick);
        other.seed = 3;
        assert_ne!(a.cache_key(), TextureSource::Procedural(other).cache_key());
    }

    #[test]
    fn json_uses_a_readable_tag() {
        let json = serde_json::to_string(&TextureRef::file("x.png")).unwrap();
        assert!(json.contains("\"type\":\"file\""), "{json}");
        let back: TextureRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back, TextureRef::file("x.png"));
    }
}
