//! Parser of IES LM-63 photometric files (1986, 1991, 1995 and 2002 editions).
//!
//! Layout: an optional `IESNA:` version line, `[KEYWORD] value` lines, a `TILT=` line, then whitespace- or
//! comma-separated numbers that may wrap freely across lines: the optional tilt block (`TILT=INCLUDE`), ten luminaire
//! values, three electrical values, the vertical angles, the horizontal angles and the candela table (one run of
//! vertical values per horizontal angle). A `TILT=<file>` reference is accepted and ignored: the lamp tilt only
//! changes the output of lamps mounted at other angles than the test.

use crate::features::lighting::models::{IesProfile, PhotometricType};

/// Upper bound on angle counts, against corrupt files.
const MAX_ANGLES: usize = 4096;

/// Why an IES file could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IesError {
    /// No `TILT=` line.
    MissingTilt,
    /// The numeric block ended early.
    Truncated,
    /// A value is not a number.
    InvalidNumber,
    /// Angle counts or photometric type out of range.
    BadHeader,
    /// Angles are not in ascending order.
    UnsortedAngles,
}

impl IesError {
    /// Localization key describing the error.
    pub fn message_key(self) -> &'static str {
        match self {
            Self::MissingTilt => "lighting.ies_error_tilt",
            Self::Truncated => "lighting.ies_error_truncated",
            Self::InvalidNumber => "lighting.ies_error_number",
            Self::BadHeader => "lighting.ies_error_header",
            Self::UnsortedAngles => "lighting.ies_error_angles",
        }
    }
}

/// Sequential reader of the numeric block.
struct Numbers<'a> {
    tokens: std::vec::IntoIter<&'a str>,
}

impl<'a> Numbers<'a> {
    fn new(lines: &[&'a str]) -> Self {
        let tokens: Vec<&'a str> = lines
            .iter()
            .flat_map(|line| line.split(|c: char| c.is_whitespace() || c == ','))
            .filter(|token| !token.is_empty())
            .collect();
        Self {
            tokens: tokens.into_iter(),
        }
    }

    fn next(&mut self) -> Result<f32, IesError> {
        let token = self.tokens.next().ok_or(IesError::Truncated)?;
        let value: f32 = token.parse().map_err(|_| IesError::InvalidNumber)?;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(IesError::InvalidNumber)
        }
    }

    fn count(&mut self) -> Result<usize, IesError> {
        let value = self.next()?;
        if value < 0.0 || value.fract() != 0.0 || value as usize > MAX_ANGLES {
            return Err(IesError::BadHeader);
        }
        Ok(value as usize)
    }

    fn take(&mut self, count: usize) -> Result<Vec<f32>, IesError> {
        (0..count).map(|_| self.next()).collect()
    }
}

fn keyword_value<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = line.trim_start().strip_prefix('[')?;
    let (name, value) = rest.split_once(']')?;
    name.eq_ignore_ascii_case(keyword).then(|| value.trim())
}

fn ascending(values: &[f32]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

/// Parses the text of an IES LM-63 file.
pub fn parse_ies(text: &str) -> Result<IesProfile, IesError> {
    let lines: Vec<&str> = text.lines().collect();
    let tilt_at = lines
        .iter()
        .position(|line| {
            line.trim_start()
                .get(..5)
                .is_some_and(|head| head.eq_ignore_ascii_case("TILT="))
        })
        .ok_or(IesError::MissingTilt)?;
    let header = &lines[..tilt_at];
    let label = ["LUMINAIRE", "TEST", "LUMCAT"]
        .iter()
        .find_map(|keyword| header.iter().find_map(|line| keyword_value(line, keyword)))
        .unwrap_or_default()
        .to_owned();
    let tilt = lines[tilt_at].trim()[5..].trim().to_ascii_uppercase();
    let mut numbers = Numbers::new(&lines[tilt_at + 1..]);
    if tilt == "INCLUDE" {
        numbers.next()?; // lamp-to-luminaire geometry
        let pairs = numbers.count()?;
        numbers.take(pairs * 2)?;
    }
    let lamps = numbers.count()?;
    let lumens_per_lamp = numbers.next()?;
    let multiplier = numbers.next()?;
    let vertical_count = numbers.count()?;
    let horizontal_count = numbers.count()?;
    let photometric_type = match numbers.next()? as i32 {
        1 => PhotometricType::C,
        2 => PhotometricType::B,
        3 => PhotometricType::A,
        _ => return Err(IesError::BadHeader),
    };
    numbers.take(4)?; // units, width, length, height
    let ballast = numbers.next()?;
    let ballast_lamp = numbers.next()?;
    numbers.next()?; // input watts
    if vertical_count == 0 || horizontal_count == 0 {
        return Err(IesError::BadHeader);
    }
    let vertical = numbers.take(vertical_count)?;
    let horizontal = numbers.take(horizontal_count)?;
    if !ascending(&vertical) || !ascending(&horizontal) {
        return Err(IesError::UnsortedAngles);
    }
    let factor = multiplier * ballast * ballast_lamp;
    let candela = (0..horizontal_count)
        .map(|_| {
            numbers
                .take(vertical_count)
                .map(|row| row.into_iter().map(|cd| (cd * factor).max(0.0)).collect())
        })
        .collect::<Result<Vec<Vec<f32>>, IesError>>()?;
    Ok(IesProfile {
        label,
        photometric_type,
        lumens_per_lamp,
        lamps: lamps as u32,
        vertical,
        horizontal,
        candela,
    })
}
