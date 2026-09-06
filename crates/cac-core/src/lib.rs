mod date;
mod document;
mod rich_text;

pub use date::{DatePoint, Period, PeriodError};
pub use document::{
    Contact, CustomEntry, CvDocument, EducationEntry, Entry, EntryKind, ExperienceEntry, Origin,
    Profile, ProjectEntry, PublicationEntry, Section, SectionKind, SkillGroupEntry, TagSet,
    TextEntry,
};
pub use rich_text::{Inline, RichText, RichTextError, escape_markdown, supported_link};

pub type ResolvedCv = CvDocument;
