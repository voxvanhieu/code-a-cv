//! The C3 acceptance test: `contract.typ` compiles and runs with plain
//! Typst, outside the cac virtual world — no `/.cac` imports, no injected
//! settings. Editors and `typst watch` can check layout code against it.

use typst::diag::FileResult;
use typst::foundations::Bytes;
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::fonts::FontStore;
use typst_layout::PagedDocument;

const CONTRACT_TYP: &str = include_str!("../src/typst/contract.typ");

const MAIN_TYP: &str = r##"
#import "/contract.typ": layout, format-date, format-period

// The formatter runs on structured dates — dict form and the canonical
// strings the view actually serializes — never on display text.
#assert(format-date("en", (y: 2023)) == "2023", message: "year only")
#assert(format-date("en", (y: 2023, m: 1)) == "January 2023", message: "year month en")
#assert(format-date("en", (y: 2023, m: 1, d: 15)) == "January 15, 2023", message: "full date en")
#assert(format-date("de", (y: 2023, m: 3)) == "März 2023", message: "year month de")
#assert(format-date("ja", (y: 2023, m: 8)) == "2023年8月", message: "year month ja")
#assert(format-date("vi", (y: 2022, m: 10)) == "Tháng Mười 2022", message: "year month vi")
#assert(format-date("en", "present") == "Present", message: "present en")
#assert(format-date("en", "Present") == "Present", message: "present capitalized")
#assert(format-date("ja", "present") == "現在", message: "present ja")
#assert(format-date("unknown", (y: 2023, m: 1)) == "January 2023", message: "unknown locale falls back to English")
#assert(format-date("en", "2023") == "2023", message: "string year")
#assert(format-date("en", "2023-01") == "January 2023", message: "string year month")
#assert(format-date("en", "2023-01-15") == "January 15, 2023", message: "string full date")
#assert(format-date("ja", "2023-08") == "2023年8月", message: "string year month ja")

// Periods render with the en dash and omit absent endpoints.
#assert(format-period("en", (start: (y: 2023, m: 1), end: "present")) == "January 2023 – Present", message: "period range")
#assert(format-period("en", (start: "2023-01", end: "Present")) == "January 2023 – Present", message: "period from strings")
#assert(format-period("en", (start: none, end: (y: 2024))) == "2024", message: "period end only")
#assert(format-period("en", (start: none, end: none)) == none, message: "period empty")

// The layout table exposes all four hooks.
#assert("document" in layout and "header" in layout and "section" in layout and "entry" in layout, message: "hook table")

// An entry renders from its named slots.
#let sample = (
  kind: "experience",
  title: ((kind: "text", text: "Director"),),
  subtitle: ((kind: "text", text: "Analytical Engines"),),
  period: (start: (y: 2023, m: 1), end: "present"),
  date: none,
  location: none,
  url: none,
  fields: (:),
  highlights: (),
  content: none,
  tags: (),
)
#let ctx = (style: (locale: "en"), layout: layout)
#(layout.entry)(ctx, sample)
"##;

struct ContractWorld {
    library: LazyHash<Library>,
    fonts: FontStore,
    main: FileId,
    main_source: Source,
}

impl ContractWorld {
    fn new() -> Self {
        let main = FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("main.typ").expect("valid main path"),
        ));
        let mut fonts = FontStore::new();
        fonts.extend(typst_kit::fonts::embedded());
        Self {
            library: LazyHash::new(Library::default()),
            fonts,
            main,
            main_source: Source::new(main, MAIN_TYP.into()),
        }
    }

    fn contract_id() -> FileId {
        FileId::new(RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new("contract.typ").expect("valid contract path"),
        ))
    }
}

impl World for ContractWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            return Ok(self.main_source.clone());
        }
        if id.vpath().get_without_slash() == "contract.typ" {
            return Ok(Source::new(Self::contract_id(), CONTRACT_TYP.to_string()));
        }
        Err(typst::diag::FileError::NotFound("contract.typ".into()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(typst::diag::FileError::NotFound(
            id.vpath().get_without_slash().to_string().into(),
        ))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(
        &self,
        _: Option<typst::foundations::Duration>,
    ) -> Option<typst::foundations::Datetime> {
        // Deterministic builds refuse the clock, even in contract tests.
        None
    }
}

#[test]
fn contract_typ_compiles_and_runs_outside_cac() {
    let world = ContractWorld::new();
    let compiled = typst::compile::<PagedDocument>(&world);
    let document = compiled.output.unwrap();
    let warns = compiled.warnings;
    let warned = warns
        .iter()
        .map(|warning| warning.message.to_string())
        .collect::<Vec<_>>();
    assert!(
        warned.is_empty(),
        "contract.typ compiles without warnings: {warned:?}"
    );
    assert!(!document.pages().is_empty(), "the sample renders one page");
}
