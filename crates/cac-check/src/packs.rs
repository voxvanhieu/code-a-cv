//! Pack validation: open fields checked against declared packs, with
//! coverage for required fields and advisories for undeclared keys.
//!
//! Required fields resolve against the model, not the pack's spelling:
//!
//! - A required **profile** key is missing when the profile has neither the
//!   typed slot for a core key (`photo`, `email`, `phone`, `location`,
//!   `website`, `summary`, `headline`, `address`) nor a matching open field
//!   ([`cac_core::Profile::carries_core_field`]).
//! - A required **entry** key is checked only on entries that can carry it.
//!   A core key uses the core field table's entry scopes, so a required
//!   `organization` flags experience entries only. A pack-only key is
//!   required on an entry when the entry's section already uses the key —
//!   at least one sibling entry declares it. A locale pack can therefore
//!   require `commute` without firing on sections that never carry it,
//!   while an entry that skips a field its siblings carry is an error.
//!
//! Identical diagnostics are reported once, however many packs agree.

use std::collections::BTreeSet;

use crate::{Diagnostic, Severity};
use cac_core::{CvDocument, FieldPack, PackScope, core_field, validate_field_value};

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
    for pack in packs {
        for (key, def) in &pack.profile {
            if def.required && !cv.profile.carries_core_field(key) {
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
                    if !def.required
                        || entry.fields.get(key).is_some()
                        || entry.carries_core_field(key)
                    {
                        continue;
                    }
                    let applies = match core_field(key) {
                        // Core key: every entry whose kind can carry it.
                        Some(field) => field.applies_to_entry_kind(entry.kind.kind_name()),
                        // Pack-only key: the section already uses it.
                        None => section
                            .entries
                            .iter()
                            .any(|sibling| sibling.fields.get(key).is_some()),
                    };
                    if applies {
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
    // Several packs can agree on the same problem; report it once.
    let mut seen = BTreeSet::new();
    output.retain(|diagnostic| {
        seen.insert((
            diagnostic.code,
            diagnostic.severity,
            diagnostic.message.clone(),
            diagnostic.path.clone(),
        ))
    });
    output
}
