//! Schema migrations: a chain of steps that upgrades an old JSON document one version at a time.

use serde_json::Value;

/// One migration, upgrading a document from `from` to `from + 1` in place.
pub struct Step {
    /// Schema version this step upgrades from.
    pub from: u32,
    /// The upgrade.
    pub apply: fn(&mut Value),
}

/// Migrations of the real schema. Schema 1 is the first version, so there are none yet; add a step (and a test)
/// whenever `SCHEMA_VERSION` grows.
pub const MIGRATIONS: &[Step] = &[];

/// Upgrades `document` from `from` to `to` using `steps`; returns the versions it passed through.
pub fn migrate(
    document: &mut Value,
    from: u32,
    to: u32,
    steps: &[Step],
) -> Result<Vec<u32>, String> {
    let mut applied = Vec::new();
    for version in from..to {
        let step = steps
            .iter()
            .find(|step| step.from == version)
            .ok_or_else(|| {
                format!(
                    "falta la migración del esquema {version} al {}",
                    version + 1
                )
            })?;
        (step.apply)(document);
        applied.push(version + 1);
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rename_title(value: &mut Value) {
        let title = value["title"].take();
        value["meta"] = json!({ "name": title });
    }

    fn add_fps(value: &mut Value) {
        value["fps"] = json!(30);
    }

    #[test]
    fn steps_run_in_order_from_the_stored_version() {
        let table = [
            Step {
                from: 1,
                apply: rename_title,
            },
            Step {
                from: 2,
                apply: add_fps,
            },
        ];
        let mut document = json!({ "title": "Casa" });
        assert_eq!(migrate(&mut document, 1, 3, &table), Ok(vec![2, 3]));
        assert_eq!(document["meta"]["name"], "Casa");
        assert_eq!(document["fps"], 30);
    }

    #[test]
    fn starting_midway_skips_earlier_steps() {
        let table = [
            Step {
                from: 1,
                apply: rename_title,
            },
            Step {
                from: 2,
                apply: add_fps,
            },
        ];
        let mut document = json!({ "meta": { "name": "x" } });
        assert_eq!(migrate(&mut document, 2, 3, &table), Ok(vec![3]));
        assert!(document.get("fps").is_some());
    }

    #[test]
    fn a_missing_step_is_an_error() {
        let mut document = json!({});
        assert!(migrate(
            &mut document,
            1,
            3,
            &[Step {
                from: 1,
                apply: add_fps
            }]
        )
        .is_err());
    }

    #[test]
    fn the_real_table_covers_every_version() {
        let mut document = json!({});
        assert!(migrate(
            &mut document,
            crate::features::projects::models::SCHEMA_VERSION,
            crate::features::projects::models::SCHEMA_VERSION,
            MIGRATIONS
        )
        .is_ok());
    }
}
