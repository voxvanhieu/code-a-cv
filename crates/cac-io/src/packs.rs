//! Field-pack loading: parse a fields.toml and resolve pack names from
//! front matter declarations.

use cac_core::{FieldPack, PackFieldDef, PackType};
use std::collections::BTreeMap;
use std::path::Path;

use crate::ParseError;

/// Parse a pack's TOML source into a named [`FieldPack`].
///
/// The TOML shape: `[fields.<scope>.<key>]` tables with `label`, `type`,
/// and optional `required`.
pub fn load_pack_from_str(name: &str, source: &str) -> Result<FieldPack, ParseError> {
    let raw: toml::Value = toml::from_str(source).map_err(|error| ParseError::Markdown {
        line: 1,
        message: format!("invalid pack `{name}`: {error}"),
    })?;
    let mut pack = FieldPack {
        name: name.to_string(),
        profile: BTreeMap::new(),
        entry: BTreeMap::new(),
    };
    let Some(fields) = raw.get("fields").and_then(|value| value.as_table()) else {
        return Err(ParseError::Markdown {
            line: 1,
            message: format!("pack `{name}` has no [fields] table"),
        });
    };
    for (scope_name, scope) in fields {
        let target = match scope_name.as_str() {
            "profile" => &mut pack.profile,
            "entry" => &mut pack.entry,
            other => {
                return Err(ParseError::Markdown {
                    line: 1,
                    message: format!("pack `{name}` scope `{other}` must be profile or entry"),
                });
            }
        };
        let Some(table) = scope.as_table() else {
            continue;
        };
        for (key, def) in table {
            let def_table = def.as_table().ok_or_else(|| ParseError::Markdown {
                line: 1,
                message: format!("pack `{name}` field `{key}` must be a table"),
            })?;
            let label = def_table
                .get("label")
                .and_then(|value| value.as_str())
                .ok_or_else(|| ParseError::Markdown {
                    line: 1,
                    message: format!("pack `{name}` field `{key}` needs a label"),
                })?
                .to_string();
            let kind = match def_table.get("type").and_then(|value| value.as_str()) {
                Some("text") | None => PackType::Text,
                Some("integer") => PackType::Integer,
                Some("boolean") => PackType::Boolean,
                Some("length") => PackType::Length,
                Some(other) => {
                    return Err(ParseError::Markdown {
                        line: 1,
                        message: format!(
                            "pack `{name}` field `{key}` type `{other}` must be text, integer, boolean, or length"
                        ),
                    });
                }
            };
            let required = def_table
                .get("required")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            target.insert(
                key.to_ascii_lowercase(),
                PackFieldDef {
                    label,
                    kind,
                    required,
                },
            );
        }
    }
    Ok(pack)
}

/// Load a pack file from disk.
pub fn load_pack_file(path: &Path) -> Result<FieldPack, ParseError> {
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("pack")
        .to_string();
    let source = std::fs::read_to_string(path).map_err(|error| ParseError::Markdown {
        line: 1,
        message: format!("could not read pack `{}`: {error}", path.display()),
    })?;
    load_pack_from_str(&name, &source)
}
