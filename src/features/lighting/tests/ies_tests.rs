//! IES LM-63 parsing and lookup.

use super::close;
use crate::features::lighting::logic::{ies_intensity, parse_ies, IesError};
use crate::features::lighting::models::PhotometricType;

/// A symmetric type C downlight: 1000 cd at the nadir falling to 0 at 90 degrees. Numbers wrap over several lines.
pub const DOWNLIGHT: &str = "IESNA:LM-63-2002\n\
[TEST] ABC123\n\
[MANUFAC] Guanacaste Test\n\
[LUMINAIRE] Downlight de prueba\n\
TILT=NONE\n\
1 1000 1 5 1 1 2 0.1 0.1 0.0\n\
1.0 1.0 12.5\n\
0 22.5 45 67.5 90\n\
0\n\
1000 900, 500\n\
200 0\n";

#[test]
fn parses_header_and_table() {
    let profile = parse_ies(DOWNLIGHT).expect("valid file");
    assert_eq!(profile.label, "Downlight de prueba");
    assert_eq!(profile.photometric_type, PhotometricType::C);
    assert_eq!(profile.lamps, 1);
    assert_eq!(profile.lumens_per_lamp, 1000.0);
    assert_eq!(profile.vertical, vec![0.0, 22.5, 45.0, 67.5, 90.0]);
    assert_eq!(profile.horizontal, vec![0.0]);
    assert_eq!(
        profile.candela,
        vec![vec![1000.0, 900.0, 500.0, 200.0, 0.0]]
    );
    assert_eq!(profile.max_candela(), 1000.0);
}

#[test]
fn intensity_interpolates_between_angles() {
    let profile = parse_ies(DOWNLIGHT).expect("valid file");
    close(ies_intensity(&profile, [0.0, 0.0, 1.0]), 1000.0, 1e-4);
    close(ies_intensity(&profile, [0.0, 0.0, 5.0]), 1000.0, 1e-4);
    let half = std::f32::consts::FRAC_1_SQRT_2;
    close(ies_intensity(&profile, [half, 0.0, half]), 500.0, 1e-3);
    // Rotationally symmetric: the horizontal angle does not matter.
    close(ies_intensity(&profile, [0.0, half, half]), 500.0, 1e-3);
    let (s, c) = 33.75_f32.to_radians().sin_cos();
    close(ies_intensity(&profile, [s, 0.0, c]), 700.0, 1e-3);
    assert_eq!(ies_intensity(&profile, [1.0, 0.0, 0.0]), 0.0);
    assert_eq!(ies_intensity(&profile, [0.0, 0.0, -1.0]), 0.0);
    assert_eq!(ies_intensity(&profile, [0.0, 0.0, 0.0]), 0.0);
}

#[test]
fn multiplier_and_ballast_scale_candela() {
    let text = "TILT=NONE\n1 1000 2 2 1 1 1 0 0 0\n0.5 1 10\n0 90\n0\n100 50\n";
    let profile = parse_ies(text).expect("valid file");
    assert_eq!(profile.candela, vec![vec![100.0, 50.0]]);
}

#[test]
fn quadrant_symmetry_is_folded() {
    let text = "TILT=NONE\n1 1000 1 2 3 1 1 0 0 0\n1 1 10\n0 90\n0 45 90\n100 0\n80 0\n20 0\n";
    let profile = parse_ies(text).expect("valid file");
    // Horizontal 0 deg is the +X axis, 90 deg the +Y axis; the file measured one quadrant.
    close(ies_intensity(&profile, [0.0, 0.0, 1.0]), 100.0, 1e-4);
    let at = |x: f32, y: f32| ies_intensity(&profile, [x, y, 0.3]);
    close(at(1.0, 0.0), at(-1.0, 0.0), 1e-4);
    close(at(0.0, 1.0), at(0.0, -1.0), 1e-4);
    assert!(at(1.0, 0.0) > at(0.0, 1.0));
}

#[test]
fn skips_tilt_block_and_accepts_lowercase_keywords() {
    let text = "[luminaire] Con inclinación\nTILT=INCLUDE\n1\n2\n0 90\n1 0.5\n\
1 1000 1 2 1 1 1 0 0 0\n1 1 10\n0 90\n0\n10 20\n";
    let profile = parse_ies(text).expect("valid file");
    assert_eq!(profile.label, "Con inclinación");
    assert_eq!(profile.candela, vec![vec![10.0, 20.0]]);
}

#[test]
fn rejects_corrupt_files() {
    assert_eq!(parse_ies("sin cabecera"), Err(IesError::MissingTilt));
    assert_eq!(
        parse_ies("TILT=NONE\n1 1000 1 2 1 1 1 0 0 0\n1 1 10\n0 90\n0\n10\n"),
        Err(IesError::Truncated)
    );
    assert_eq!(
        parse_ies("TILT=NONE\n1 mil 1 2 1 1 1 0 0 0\n"),
        Err(IesError::InvalidNumber)
    );
    assert_eq!(
        parse_ies("TILT=NONE\n1 1000 1 2 1 9 1 0 0 0\n1 1 10\n0 90\n0\n10 20\n"),
        Err(IesError::BadHeader)
    );
    assert_eq!(
        parse_ies("TILT=NONE\n1 1000 1 2 1 1 1 0 0 0\n1 1 10\n90 0\n0\n10 20\n"),
        Err(IesError::UnsortedAngles)
    );
    assert!(IesError::Truncated.message_key().starts_with("lighting."));
}
