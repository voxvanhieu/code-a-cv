mod date;
mod document;
mod fields;
mod packs;
mod rich_text;

pub use date::{DatePoint, Period, PeriodError};
pub use document::{
    Address, Contact, CustomEntry, CvDocument, EducationEntry, Entry, EntryKind, ExperienceEntry,
    LanguageEntry, Origin, Profile, ProjectEntry, PublicationEntry, ReferenceEntry, Section,
    SectionKind, SkillGroupEntry, TagSet, TextEntry,
};
pub use fields::{
    CORE_FIELDS, EntryKindName, FieldDef, FieldMap, FieldParser, FieldScope, FieldValue, OpenField,
    Skill, SkillLevel, core_field, normalize_key,
};
pub use packs::{FieldPack, PackFieldDef, PackScope, PackType, validate_field_value};
pub use rich_text::{Inline, RichText, RichTextError, escape_markdown, supported_link};

pub type ResolvedCv = CvDocument;
