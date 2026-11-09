//! YAML front matter: declarations and profile fields (rule D).
//!
//! Every Markdown tool preserves front matter, so it is the standard place
//! for declarations — `locale` and `packs` — alongside profile fields.
//! Declaration consumption lands with the field-pack task (M2); the shapes
//! validate now so documents that use them parse unchanged.

use super::markdown_error;
use crate::ParseError;

pub(super) struct FrontMatter {
    pub locale: Option<String>,
    pub packs: Vec<String>,
}

pub(super) struct FrontMatterField {
    pub key: String,
    pub value: String,
    pub line: usize,
}

pub(super) fn parse(
    raw: &str,
    line: usize,
) -> Result<(FrontMatter, Vec<FrontMatterField>), ParseError> {
    let value: serde_json::Value =
        serde_yaml_ng::from_str(raw).map_err(|error| ParseError::Markdown {
            line,
            message: format!("invalid front matter: {error}"),
        })?;
    let Some(map) = value.as_object() else {
        return markdown_error(line, "front matter must be a mapping of keys to values");
    };
    let mut front = FrontMatter {
        locale: None,
        packs: Vec::new(),
    };
    let mut fields = Vec::new();
    for (key, value) in map {
        let key = key.to_ascii_lowercase();
        match key.as_str() {
            "locale" => {
                front.locale = Some(scalar(value, &key, line)?);
            }
            "packs" => {
                let Some(items) = value.as_array() else {
                    return markdown_error(
                        line,
                        "front matter `packs` must be a list of pack names",
                    );
                };
                for item in items {
                    front.packs.push(scalar(item, "packs", line)?);
                }
            }
            other => {
                fields.push(FrontMatterField {
                    key: other.to_string(),
                    value: scalar(value, other, line)?,
                    line,
                });
            }
        }
    }
    Ok((front, fields))
}

fn scalar(value: &serde_json::Value, key: &str, line: usize) -> Result<String, ParseError> {
    match value {
        serde_json::Value::String(text) => Ok(text.clone()),
        serde_json::Value::Number(number) => Ok(number.to_string()),
        serde_json::Value::Bool(flag) => Ok(flag.to_string()),
        _ => markdown_error(
            line,
            &format!("front matter `{key}` must be a scalar value"),
        ),
    }
}
