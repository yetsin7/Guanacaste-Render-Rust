//! Build-mode rules: slider scaling, number formatting and guide progress.

use super::models::{BuildGuide, BuildState, SliderSpec};

/// Real value of a slider at normalized position `t`.
pub fn value_of(spec: &SliderSpec, t: f32) -> f32 {
    spec.min + (spec.max - spec.min) * t.clamp(0.0, 1.0)
}

/// Formats `value` with thousands separators and the given decimals, e.g. `1250.0` -> `"1,250"`.
pub fn with_thousands(value: f32, decimals: usize) -> String {
    let text = format!("{:.*}", decimals, value.abs());
    let (whole, fraction) = text
        .split_once('.')
        .map_or((text.as_str(), None), |(w, f)| (w, Some(f)));
    let mut grouped = String::new();
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if let Some(fraction) = fraction {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    if value < 0.0 {
        grouped.insert(0, '-');
    }
    grouped
}

/// The text shown next to a slider at normalized position `t`, e.g. `"1,250 cd"`.
pub fn format_value(spec: &SliderSpec, t: f32) -> String {
    format!(
        "{}{}",
        with_thousands(value_of(spec, t), spec.decimals),
        spec.unit
    )
}

/// Registers one more placed instance; at the end of the batch the guide moves to the next step.
pub fn place_next(state: &mut BuildState, guide: &BuildGuide) {
    if state.placed + 1 >= guide.instances_total {
        state.placed = 0;
        state.step = if state.step >= guide.step_total {
            1
        } else {
            state.step + 1
        };
    } else {
        state.placed += 1;
    }
}
