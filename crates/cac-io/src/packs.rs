//! Field-pack parsing: validate a `fields.toml`'s text into a named
//! [`FieldPack`]. The core stays filesystem-free and so does this module:
//! reading the file is the `cac` crate's job. Callers pass the source text
//! plus a path label used only in parse errors.

use std::collections::BTreeMap;
use std::fmt::{self, Formatter};

use serde::Deserialize;
use serde::de::{self, MapAccess, Visitor};

use cac_core::{FieldPack, PackFieldDef, normalize_key};

use crate::ParseError;

/// The declared pack document. Unknown keys at every level (`requird`,
/// a misspelled scope) are parse errors, not silently ignored data.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackDocument {
    name: String,
    fields: PackScopes,
}

/// The `[fields]` table: `profile` and `entry` scopes, nothing else.
#[derive(Default)]
struct PackScopes {
    profile: BTreeMap<String, PackFieldDef>,
    entry: BTreeMap<String, PackFieldDef>,
}

impl<'de> Deserialize<'de> for PackScopes {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PackScopesVisitor;

        impl<'de> Visitor<'de> for PackScopesVisitor {
            type Value = PackScopes;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a [fields] table with `profile` and `entry` scopes")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut scopes = PackScopes::default();
                while let Some(scope) = access.next_key::<String>()? {
                    match scope.as_str() {
                        "profile" => scopes.profile = access.next_value::<FieldMap>()?.0,
                        "entry" => scopes.entry = access.next_value::<FieldMap>()?.0,
                        other => {
                            return Err(de::Error::custom(format!(
                                "scope `{other}` must be `profile` or `entry`"
                            )));
                        }
                    }
                }
                Ok(scopes)
            }
        }

        deserializer.deserialize_map(PackScopesVisitor)
    }
}

/// One scope's `key = { label, type, required }` table, with the field name
/// in every error so a ten-field pack points at the broken one.
#[derive(Debug, Default)]
struct FieldMap(BTreeMap<String, PackFieldDef>);

impl<'de> Deserialize<'de> for FieldMap {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldMapVisitor;

        impl<'de> Visitor<'de> for FieldMapVisitor {
            type Value = FieldMap;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a table of field definitions")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut fields = FieldMap::default();
                while let Some(key) = access.next_key::<String>()? {
                    let def: PackFieldDef = access
                        .next_value()
                        .map_err(|error| de::Error::custom(format!("field `{key}`: {error}")))?;
                    fields.0.insert(key, def);
                }
                Ok(fields)
            }
        }

        deserializer.deserialize_map(FieldMapVisitor)
    }
}

/// Parse a pack's TOML source into a [`FieldPack`] named by the file's
/// declared `name`.
///
/// The TOML shape: `name = "..."` plus `[fields.<scope>.<key>]` tables with
/// `label`, optional `type` (default `text`), and optional `required`. Keys
/// normalize the same way CV field keys do (`commute_time` and `commute-time`
/// are one field); two spellings that collapse onto one key are an error.
pub fn parse_pack(source: &str, path: &str) -> Result<FieldPack, ParseError> {
    let invalid = |message: String| ParseError::Pack {
        path: path.to_string(),
        message,
    };
    let document: PackDocument =
        toml::from_str(source).map_err(|error| invalid(error.to_string()))?;
    let mut pack = FieldPack {
        name: document.name.clone(),
        profile: BTreeMap::new(),
        entry: BTreeMap::new(),
    };
    pack.profile = normalized_fields(document.fields.profile, &document.name, "profile", &invalid)?;
    pack.entry = normalized_fields(document.fields.entry, &document.name, "entry", &invalid)?;
    Ok(pack)
}

/// Fold declared keys through [`normalize_key`], the same folding
/// `FieldPack::field` lookups use, so a stored key is always found again.
fn normalized_fields(
    declared: BTreeMap<String, PackFieldDef>,
    pack: &str,
    scope: &str,
    invalid: &dyn Fn(String) -> ParseError,
) -> Result<BTreeMap<String, PackFieldDef>, ParseError> {
    let mut normalized = BTreeMap::new();
    let mut spellings: BTreeMap<String, String> = BTreeMap::new();
    for (key, def) in declared {
        let folded = normalize_key(&key);
        if folded.is_empty() {
            return Err(invalid(format!(
                "pack `{pack}` field `{key}` normalizes to an empty key"
            )));
        }
        if let Some(existing) = spellings.get(&folded) {
            return Err(invalid(format!(
                "pack `{pack}` {scope} keys `{existing}` and `{key}` collide under the normalized key `{folded}`"
            )));
        }
        spellings.insert(folded.clone(), key);
        normalized.insert(folded, def);
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cac_core::{PackScope, PackType};

    fn parse(source: &str) -> Result<FieldPack, ParseError> {
        parse_pack(source, "fields.toml")
    }

    #[test]
    fn declared_name_and_scopes_load() {
        let pack = parse(
            r#"
            name = "jp-rirekisho"

            [fields.profile.photo]
            label = "Photo"
            required = true

            [fields.entry.commute]
            label = "Commute"
            type = "text"

            [fields.entry.available_from]
            label = "Available from"
            type = "date"
            "#,
        )
        .unwrap();
        assert_eq!(pack.name, "jp-rirekisho", "the declared name wins");
        assert!(pack.profile["photo"].required);
        assert_eq!(pack.entry["commute"].kind, PackType::Text);
        assert_eq!(pack.entry["available-from"].kind, PackType::Date);
    }

    #[test]
    fn keys_normalize_and_collisions_are_errors() {
        let pack = parse(
            r#"
            name = "jp"
            [fields.entry.commute_time]
            label = "Commute"
            [fields.entry."Commute Time"]
            label = "Commute"
            "#,
        );
        assert!(
            matches!(&pack, Err(ParseError::Pack { message, .. })
                if message.contains("collide") && message.contains("commute-time")),
            "colliding spellings error: {pack:?}"
        );
        let pack = parse(
            r#"
            name = "jp"
            [fields.entry.Commute_Time]
            label = "Commute"
            "#,
        )
        .unwrap();
        assert!(
            pack.field(PackScope::Entry, "commute-time").is_some(),
            "`commute_time` and `commute-time` are one field"
        );
    }

    #[test]
    fn unknown_keys_and_scopes_are_errors() {
        for source in [
            "name = \"jp\"\nrequird = true\n[fields.entry.a]\nlabel = \"A\"",
            "nam = \"jp\"\n[fields.entry.a]\nlabel = \"A\"",
            "name = \"jp\"\n[fields.prfile.a]\nlabel = \"A\"",
            "name = \"jp\"\n[fields]\nprofile = \"text\"",
            "name = \"jp\"\n[fields.entry.a]\nlabel = \"A\"\nrequird = true",
            "name = \"jp\"\n[fields.entry.a]\ntype = \"nmber\"",
            "name = \"jp\"\n[fields.entry]\na = 5",
        ] {
            assert!(
                matches!(parse(source), Err(ParseError::Pack { .. })),
                "strict parse rejects: {source}"
            );
        }
    }

    #[test]
    fn errors_carry_the_path_label() {
        let error =
            parse_pack("[fields.entry.a]\nlabel = \"A\"", "packs/jp/fields.toml").unwrap_err();
        let rendered = error.to_string();
        assert!(
            rendered.starts_with("invalid field pack at packs/jp/fields.toml:")
                && rendered.contains("missing field `name`"),
            "path label and cause both render: {rendered}"
        );
    }

    #[test]
    fn field_errors_name_the_field() {
        let error = parse("name = \"jp\"\n[fields.entry.commute]\ntype = \"text\"\nrequird = true")
            .unwrap_err();
        assert!(
            error.to_string().contains("field `commute`"),
            "the broken field is named: {error}"
        );
    }
}
