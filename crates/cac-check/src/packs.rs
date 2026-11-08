//! Pack validation: open fields checked against declared packs, with
//! coverage for required fields and advisories for undeclared keys.

use crate::{Diagnostic, Severity};
use cac_core::{CvDocument, FieldPack, PackScope, validate_field_value};

/// Validate a document's open fields against the loaded packs. Undeclared
/// keys are advisories; mistyped declared values and missing required
/// fields are errors.
pub fn check_packs(cv: &CvDocument, packs: &[FieldPack]) -> Vec<Diagnostic> {
    if packs.is_empty() {
        return Vec::new();
    }
    let mut output = Vec::new();
    fn declare(
        output: &mut Vec<Diagnostic>,
        packs: &[FieldPack],
        scope: PackScope,
        path: &str,
        key: &str,
        value: &cac_core::FieldValue,
    ) {
        let mut matching = Vec::new();
        for pack in packs {
            if let Some(def) = pack.field(scope, key) {
                matching.push((def, pack.name.clone()));
            }
        }
        if matching.is_empty() {
            output.push(Diagnostic {
                code: "CACPACK1",
                severity: Severity::Warning,
                message: format!(
                    "open field `{key}` is not declared by any loaded pack; declare it in fields.toml"
                ),
                path: path.to_string(),
            });
            return;
        }
        for (def, pack) in matching {
            if let Err(message) = validate_field_value(value, &def.kind) {
                output.push(Diagnostic {
                    code: "CACPACK2",
                    severity: Severity::Error,
                    message: format!("pack `{pack}` field `{key}`: {message}"),
                    path: path.to_string(),
                });
            }
        }
    }

    for field in cv.profile.fields.iter() {
        declare(
            &mut output,
            packs,
            PackScope::Profile,
            "profile.fields",
            &field.key.clone(),
            &field.value.clone(),
        );
    }
    // Required profile fields resolve through the model keys they map to:
    // `image` is the photo, `label` the headline, everything else an open
    // field on the profile.
    for pack in packs {
        for (key, def) in &pack.profile {
            if !def.required {
                continue;
            }
            let missing = match key.as_str() {
                "image" => cv.profile.photo.is_none(),
                "label" => cv.profile.headline.is_none(),
                other => cv.profile.fields.get(other).is_none(),
            };
            if missing {
                output.push(Diagnostic {
                    code: "CACPACK3",
                    severity: Severity::Error,
                    message: format!(
                        "pack `{}` requires profile `{}` ({})",
                        pack.name, key, def.label
                    ),
                    path: "profile".to_string(),
                });
            }
        }
    }
    for (section_index, section) in cv.sections.iter().enumerate() {
        for (entry_index, entry) in section.entries.iter().enumerate() {
            let path = format!("sections[{section_index}].entries[{entry_index}]");
            for field in entry.fields.iter() {
                declare(
                    &mut output,
                    packs,
                    PackScope::Entry,
                    &path,
                    &field.key.clone(),
                    &field.value.clone(),
                );
            }
            for pack in packs {
                for (key, def) in &pack.entry {
                    if def.required
                        && entry.fields.get(key).is_none()
                        && section
                            .entries
                            .iter()
                            .all(|sibling| sibling.fields.get(key).is_none())
                    {
                        output.push(Diagnostic {
                            code: "CACPACK3",
                            severity: Severity::Error,
                            message: format!(
                                "pack `{}` requires entry field `{}` ({})",
                                pack.name, key, def.label
                            ),
                            path: path.clone(),
                        });
                    }
                }
            }
        }
    }
    output
}
