# `cac` — CV as Code

Implementation plan · v0.2 · August 2026

---

## 1. What this is

A CLI that compiles Markdown-first CV data into a high-quality PDF. One static binary, no runtime, no TeX.

The positioning in one line: **everything in this space needs either a Python runtime or a 4 GB TeX install. This doesn't.**

Three secondary bets, all opt-in flags so they cost nothing to a user who ignores them:

- `--select` — one master CV, tag expressions produce targeted versions
- `--fit` — compress to a page budget automatically
- `cac check` — verify the emitted PDF's text layer actually contains your content

**~6 weeks to 1.0.** First usable release at week 2.

---

## 2. Landscape

### `cv-cli` (the reference project)

YAML → Jinja2 → LaTeX → external `pdflatex`. Needs Python **and** a TeX distribution. Three commands (`build`, `profiles`, `templates`). Templates derive from Jake Gutierrez's LaTeX resume.

Two structural weaknesses worth naming, because they define the wedge:

- **The TeX dependency is the dominant install-friction cost.** 2–5 GB, a notorious failure point on Windows and in CI containers.
- **Jinja2 over LaTeX is an escaping minefield.** `& % $ # _ { } ~ ^ \` all need context-dependent escaping. Any user with `C#`, `R&D`, or `~/path` in their CV hits it. Every LaTeX-templating tool carries a permanent bug tail here.

Also: `templates clone <url>` executes arbitrary LaTeX, and LaTeX has `\write18`.

### RenderCV — the actual competitor (~17k stars)

The serious incumbent. It made the right architectural call first: **migrated from LaTeX to Typst**, and the community reaction was strongly positive.

- Pipeline: YAML → generated `.typ` → Typst → PDF. Also PNG, HTML, Markdown.
- Pydantic validation + generated JSON Schema for editor autocomplete.
- **9 themes**, **22 locales** (including Vietnamese, Arabic, Hebrew, Japanese, Korean, Mandarin — RTL and CJK are handled).
- `--watch`, dot-notation overrides (`--cv.phone "…"`), an AI agent skill auto-generated from the Pydantic models.
- Published an ATS study: 2 extractors + 3 commercial parsers over 20 PDFs. Residual error was mostly **typographic** (curly quotes), not missing content.

**Its entire CLI is three commands: `new`, `render`, `create-theme`.** No lint, no check, no format, no standalone validate.

That gap is precise, and worth being careful about:

| Layer | RenderCV | `cac` |
|---|---|---|
| **Validate** — well-formed? | Yes, and good. Nested Pydantic + JSON Schema. | Same, via serde + schemars |
| **Lint** — is the content good? | Nothing. | `cac check`, content phase |
| **Check** — did it survive rendering? | Not as a command. The ATS work is a published study, not a per-build assertion. | `cac check`, rendered phase |

**Where not to claim differentiation.** RenderCV is at parity or ahead on theme count, locales, watch mode, export formats, editor schema, and agent tooling. Those are **table stakes to reach**, not features to lead with. Reaching parity is most of the work; the differentiators are the smaller slice.

### JSON Resume

Open schema (`basics`, `work`, `education`, `skills`…), hundreds of community themes, a hosted registry. Don't compete with it — **consume and emit it**. One adapter buys a migration path in and an escape hatch out.

### Why Typst

- Rust, Apache-2.0, **published on crates.io as an embeddable library**. Implement the `World` trait and you control its filesystem, fonts, and clock. No subprocess, no external binary.
- **0.14 emits tagged PDFs by default**, plus PDF/UA-1 and the full PDF/A range. Tagged PDF is exactly the structure machine parsers want — the accessibility win and the ATS win are the same win.
- **0.15** (June 2026) added variable fonts and MathML.
- Compiles a CV in tens of milliseconds. That's what makes `--fit` viable.
- Sandboxed: a theme can't read your filesystem or shell out.

**Caveat:** pre-1.0, breaking changes each minor release. Mitigation in §12.

---

## 3. Scope

### In

| | Notes |
|---|---|
| Single static binary, embedded Typst | The reason the project exists |
| Markdown input by default | Human-readable source that mirrors the CV's visible sections. §5.2 |
| YAML / JSON / TOML input | Supported structured formats for automation and conversion |
| `RichText` inline AST | Internal invariant, not a feature. Kills the escaping bug class. §5.2 |
| JSON Schema for structured formats | Editor completion and validation for YAML and JSON. §5.4 |
| PDF + HTML output | HTML is one generic, theme-independent template |
| 2 themes | Two that are genuinely good, not nine that are fine |
| JSON Resume import/export | One adapter |
| `--select` tag expressions | §7 |
| `--fit` page budget | Typography only, no content deletion. §8 |
| `cac check` | 12 rules, 2 phases, incl. ATS round-trip. §9 |
| `cac watch` | Re-render on save |
| Reproducible builds | Pinned timestamp + PDF ID. Invisible to users, ~free |
| `wasm32` build target | §11 |

### Out, and why

The filter: a CV tool is opened **5–20 times a year, under deadline pressure**. Not a daily driver. So — *does it cost anything if you never use it?* A flag costs nothing. A required field, a config file, or a taxonomy to maintain costs a lot.

| Cut | Reason |
|---|---|
| **LaTeX output** | Dropped entirely, not even `--format tex`. Every line of it serves users who want the dependency we're removing. Revisit only if mandated-class-file academics prove to be a real segment. |
| **Overlays / merge-patch** | Tags + targets + overlays was three answers to one question — a sign the design wasn't finished. Tags win. |
| **Fit solver content trimming** | Needed a `priority: 0-100` field on every entry, and the behaviour was *silently deleting your bullets*. Alarming in a deadline tool. |
| **`cac match --jd`** | A 4k-term taxonomy, stemming, TF-IDF — a whole subsystem producing advisory output. Great demo, rarely used twice. |
| **Application ledger** (`apply`/`log`) | A different product. |
| **Self-describing PDFs** (source as PDF/A-3 attachment) | Clever, not needed. The source is in git. |
| **Theme registry + lockfile** | Themes are a directory or a git URL. Build a registry when you're lucky enough to need one. |
| **LSP server** | JSON Schema already gives completion, hover and validation in VS Code, Neovim, IntelliJ, Zed. |
| **`fmt`, `diff`, `doctor`, `publish`, redaction, cover letters, QR** | Low value on a file edited twice a year. `doctor` especially — with a static binary and embedded fonts there's nothing to diagnose; shipping it would admit the install is fragile. |
| Visual snapshot regression | Not a user feature — a maintainer CI concern. Moved to §13. |
| WASM **playground** | The build *target* stays. The web app is post-1.0 marketing. |

### The line to protect

```console
$ brew install code-a-cv && cac init && cac build
```

Three commands, no dependencies, a good-looking PDF. If a feature adds a required step to that line, it's P2 regardless of quality.

---

## 4. Architecture

```
cv.md   ─┐
cv.yaml ├──▶ parse ──▶ adapter ──▶ CvDocument (IR) ──▶ select ──▶ ResolvedCv
cv.json │    (Markdown | serde)  (native |                       (tags)          │
cv.toml ─┘                      jsonresume)                                    │
                                        ┌──────────────────────────────┤
                                        ▼                              ▼
                                 Typst World                    HTML template
                                 → PDF / PNG / SVG              → single-file .html
                                        │
                                        ▼
                                 check: rendered phase
                                 (re-extract text, verify)
```

**Five crates.** Fourteen would be over-decomposition for a tool this size.

```
cac-core     IR, RichText, Period, schemars derives. No I/O. wasm-clean.
cac-io       parse (markdown/yaml/json/toml) + JSON Resume adapter + diagnostics
cac-render   Typst World → PDF/PNG/SVG; HTML template
cac-check    rule engine + catalogue + ATS verification
cac-cli      clap, watch, config. The only crate touching fs/network.
```

The `cac-core` boundary is what keeps WASM cheap — everything below it must compile to `wasm32-unknown-unknown`. Enforce it in CI from week one, not week twenty.

### Crates

| Concern | Crate |
|---|---|
| CLI | `clap` v4 derive, `clap_complete` |
| Diagnostics | `miette` + `thiserror` + `serde_path_to_error` |
| YAML | `serde_yaml_ng` (`serde_yaml` is unmaintained) |
| TOML / JSON | `toml`, `serde_json` |
| Markdown | `pulldown-cmark` with a constrained CV document grammar |
| Schema | `schemars` |
| Typesetting | `typst`, `typst-pdf`, `typst-svg`, `typst-render`, `typst-kit`, `typst-assets` — pinned exactly |
| HTML templating | `minijinja` (pure Rust, autoescaping, wasm-safe) |
| PDF text extraction | `pdf-extract`, `lopdf` |
| Watch | `notify` |
| Testing | `insta`, `assert_cmd`, `proptest`, `criterion` |
| Release | `cargo-dist` (`dist`), `release-plz` |

`typst-as-lib` exists but is unofficial with an explicitly unstable API. **Implement `World` directly** so upstream churn is contained in one file you own.

---

## 5. Data model

### 5.1 IR

```rust
pub struct CvDocument {
    pub profile:  Profile,
    pub sections: Vec<Section>,
}

pub struct Section {
    pub id:      String,          // slug
    pub title:   String,
    pub kind:    SectionKind,
    pub entries: Vec<Entry>,
    pub tags:    TagSet,
}

pub struct Entry {
    #[serde(flatten)]
    pub kind: EntryKind,
    #[serde(default)]
    pub tags: TagSet,
    #[serde(skip)]
    pub origin: Origin,           // path into the source, for diagnostics
}

/// Closed set: this is what guarantees every theme renders every CV.
/// `Text` is the escape hatch.
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EntryKind {
    Experience(ExperienceEntry),
    Education(EducationEntry),
    Project(ProjectEntry),
    Publication(PublicationEntry),
    SkillGroup(SkillGroupEntry),
    Text(TextEntry),
}
```

Dates get a real type, because dates are where CV tools break:

```rust
pub enum DatePoint { Year(i32), YearMonth(i32, u8), Full(NaiveDate), Present }
pub struct Period { pub start: DatePoint, pub end: DatePoint }
```

Invariant `start <= end` validated at parse. **Never store a formatted date string** — formatting is a rendering concern driven by locale.

### 5.2 Markdown source format

`cv.md` is the default input. Its headings are the CV's headings, not a separate configuration language. `cac init` creates a reverse-chronological template based on Oxford University Careers Service guidance: contact details, Education, Experience, Additional Skills, and Interests. Education normally comes first for recent graduates; experienced candidates may place Experience first. Users can rename sections to suit the application, such as `Teaching Experience`, `Research Experience`, or `Community Activity and Skills`.

Use one level-one heading for the candidate name, level-two headings for CV sections, and level-three headings for entries. Put the entry's role or qualification first, followed by its organisation or institution and a date line. Use bullets for concise evidence of actions and results. Entries within Education and Experience are reverse chronological.

```markdown
# Ada Lovelace

ada@example.com · London, United Kingdom

## Education

### BSc Computer Science, University of London
2020–2023

- First-class degree

## Experience

### Software Engineer, Analytical Engines Ltd
Jan 2023–Present

- Reduced build time by 35% by replacing the deployment pipeline

## Additional Skills

- Rust, TypeScript, and PostgreSQL

## Interests

- Volunteer programming mentor
```

The parser treats headings, date lines, and bullet lists as structure. Markdown emphasis, links, and inline code remain rich text. Optional HTML comments carry machine-only metadata such as tags without appearing in the CV. The grammar must reject ambiguous input with a diagnostic rather than infer a meaning.

### 5.3 `RichText` — the escaping fix

The most important type in the codebase.

Never store markup as a string a backend has to escape. Store a tiny inline AST:

```rust
pub enum Inline {
    Text(String),
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Code(String),
    Link { href: Url, body: Vec<Inline> },
}
pub struct RichText(pub Vec<Inline>);
```

Parsed once from Markdown by `pulldown-cmark`. Each backend serialises it in its own syntax — Typst emits `#emph[…]`, HTML emits `<em>`, plaintext emits nothing.

**Escaping becomes structurally impossible to get wrong**, because raw text only appears inside a `Text` node whose serialiser is the one place escaping lives. This is what kills the `C#` / `100%` / `~/path` bug class permanently.

(Markdown is both the default document format and the inline rich-text syntax.)

### 5.4 Schema for structured formats

`schemars` derives a JSON Schema from the IR. YAML and JSON users can reference it for completion and inline validation:

```yaml
# yaml-language-server: $schema=https://cac.dev/schema/v1.json
profile:
  name: Christopher Nguyen
```

Autocomplete and inline validation work immediately, in every major editor, with **zero user action**. Highest-leverage item in the project for the cost.

### 5.5 Diagnostics

Don't track spans through serde. Instead: deserialise with `serde_path_to_error`, get a path (`sections[2].entries[0].period.start`), render it with `miette`. Good enough for a 200-line file. Real byte-spans for YAML are a later refinement, not a v1 requirement.

---

## 6. Rendering

### 6.1 The key decision: don't generate Typst source by string interpolation

RenderCV generates `.typ` by templating. It works, but it recreates the escaping problem one layer up and makes themes untestable in isolation.

Instead, expose the CV **as a virtual JSON file inside the `World`**:

```rust
pub struct CacWorld {
    library: LazyHash<Library>,
    book:    LazyHash<FontBook>,
    fonts:   Vec<Font>,
    vfs:     HashMap<FileId, Bytes>,   // theme files + /cv.json + assets
    main:    FileId,
    now:     Option<Datetime>,         // pinned for reproducibility
}

impl typst::World for CacWorld {
    fn library(&self) -> &LazyHash<Library> { &self.library }
    fn book(&self) -> &LazyHash<FontBook> { &self.book }
    fn main(&self) -> FileId { self.main }
    fn source(&self, id: FileId) -> FileResult<Source> {
        let b = self.vfs.get(&id).ok_or(FileError::NotFound(/*…*/))?;
        Ok(Source::new(id, String::from_utf8(b.to_vec())?))
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.vfs.get(&id).cloned().ok_or(FileError::NotFound(/*…*/))
    }
    fn font(&self, i: usize) -> Option<Font> { self.fonts.get(i).cloned() }
    fn today(&self, _: Option<i64>) -> Option<Datetime> { self.now }
}
```

The theme is then an ordinary Typst program:

```typ
#let cv = json("/cv.json")
#import "lib/blocks.typ": entry, section, header

#set document(title: cv.profile.name + " — CV", author: cv.profile.name)
#set page(paper: "a4", margin: 2cm)
#header(cv.profile)
#for s in cv.sections { section(s) }
```

Consequences, all good:

- **Zero escaping surface.** JSON encoding is total, Typst's `json()` is total. No path by which a `%` in a job title breaks the build.
- **Themes are ordinary Typst projects.** A designer drops a `cv.json` beside `main.typ`, runs `typst watch`, and iterates — no `cac` needed.
- **Incremental compilation works.** Typst's `comemo` memoises across compiles of the same `World`; only the changed `/cv.json` invalidates. This is what makes `watch` and `--fit` fast.

**Fonts.** `typst-kit` with `embed-fonts` bakes in a curated OFL/Apache set (Libertinus, Source Sans 3, DejaVu), plus system discovery via `fontdb` on native. Don't bundle Noto CJK unconditionally — that's ~40 MB on every binary. Downloadable font packs, later.

**PDF options:**

```rust
PdfOptions {
    ident: Smart::Custom(&stable_ident),
    timestamp: source_date_epoch_or_pinned(),
    standards: PdfStandards::new(&[PdfStandard::A_2b])?,
    ..Default::default()
}
```

Tagging is on by default in Typst ≥0.14.

### 6.2 HTML

**Not** Typst's HTML export — still experimental, feature-flagged, and emits no CSS.

`minijinja` over the same `ResolvedCv`. One generic, theme-independent template. Single self-contained `.html` (inlined CSS, base64 assets) so it survives being emailed. Semantic markup with `schema.org/Person` microdata, and a `@page` print stylesheet so browser Print-to-PDF is a decent cross-check.

Revisit at Typst 0.16+ if HTML export matures.

---

## 7. `--select`

Every entry can carry `tags`. Tags propagate downward — an entry inherits its section's.

```console
$ cac build --select "dotnet and not consulting"
```

A tiny boolean expression parser: identifiers, `and`, `or`, `not`, parens. Evaluated against each entry's effective tag set.

Optional named selections in `cac.toml`:

```toml
[build]
theme = "classic"

[targets]
backend  = "dotnet or backend"
academic = "research or teaching"
```

Then `cac build --target backend`. **That is the entire config surface.** No section-ordering DSL, no per-target budgets, no output-name templating.

Write no tags and nothing changes — that's what makes this free at the quick-start bar.

---

## 8. `--fit`

Typst compiles a CV in ~30–150 ms, so you can afford 5–10 compiles inside one command.

```
1. compile at theme defaults → page count P
2. if P <= budget: done
3. binary search `density ∈ [theme.min, 1.0]`, ~5 compiles
   theme maps density → leading, section spacing, entry gap, margins
4. still over budget → report the page count and name the longest sections
```

**Typography only. Never deletes content.** The earlier five-stage design trimmed highlights and entries by priority score, which required a user-visible `priority` field and meant the tool silently removed your bullets. Simpler, safer, and no schema addition.

Implementation notes:

- Keep **one `World`** across the whole solve — `comemo` reuses everything but the changed config. Iterations 2..n cost a fraction of the first.
- Page count comes from `PagedDocument::pages.len()`. Probe compiles skip `typst_pdf::pdf` entirely, which is the expensive half.
- Report what was applied: `density 1.00 → 0.93`. Never a silent change.

---

## 9. `cac check`

### One command, three phases

An earlier draft split `lint` (content) from `check --ats` (output), and contradicted itself: the layout and ATS rules *require a rendered document*, which is what `check` was supposed to own.

The axis that matters isn't quality vs. correctness. It's **cost** — some rules need only the IR (~5 ms), others need a render plus PDF re-parse (~400 ms). Cost distinctions belong in a flag, not a command name.

```console
cac check              # all phases
cac check --fast       # IR only, no render (~5 ms) — what watch runs
cac check --only CAC5xx    # ATS rules alone
cac check dist/cv.pdf      # artifact mode, no source needed
```

Follow **ruff**, not cargo. Cargo's `check`/`clippy` split is historical and reliably confuses people; ruff collapsed a dozen tools into `ruff check` and that's part of why it won. Ruff kept `ruff format` separate — the right boundary, because **`fmt` writes files and `check` doesn't**. (We cut `fmt` anyway.)

One trait, phase as data:

```rust
pub enum Phase { Content, Rendered }

pub struct Ctx<'a> {
    pub cv: &'a ResolvedCv,
    pub rendered: Option<&'a RenderedDoc>,   // None during --fast
}

pub trait Rule {
    fn code(&self) -> RuleCode;
    fn phase(&self) -> Phase;
    fn default_severity(&self) -> Severity;
    fn check(&self, ctx: &Ctx<'_>, sink: &mut DiagSink);
}
```

The engine filters by phase before dispatch, so a `Rendered` rule never defends against `None`. Promoting a rule to need the PDF is a one-line change.

### 12 rules, not 35

A rule that misfires once teaches users to ignore all of them. Ship twelve that are unambiguous.

**Content phase**

| Code | Rule |
|---|---|
| CAC101 | No contact method (neither email nor phone) |
| CAC103 | Section declared but empty |
| CAC201 | Highlight contains no number, %, or duration |
| CAC202 | Highlight opens with a weak verb (`helped`, `worked on`, `responsible for`) |
| CAC204 | Entry has zero highlights |
| CAC301 | First-person pronoun (`I`, `my`) |
| CAC603 | Inconsistent date granularity (`2021` vs `2021-03`) |

**Rendered phase — the differentiator**

| Code | Rule |
|---|---|
| CAC402 | Content exceeds the configured page budget |
| CAC404 | Glyph missing from all fonts (tofu) |
| CAC501 | A field in the IR is not recoverable from the PDF's text layer |
| CAC502 | Font not embedded |
| CAC504 | Contact info in a header/footer region parsers often skip |

**Deliberately cut:** passive voice, tense inconsistency, acronym expansion, buzzword detection, near-duplicate phrasing, date-gap analysis. Each needs linguistic heuristics that are ~80% right at best. Add them individually later, tuned against real CVs.

### The ATS check (CAC501)

Most tools *claim* ATS-friendliness. RenderCV *measured* it. `cac` can **verify it per build**, because it has something no external study has: the ground-truth IR.

1. Render the PDF.
2. Extract text with reading order (`pdf-extract`).
3. Normalise both sides: NFKC, curly → straight quotes, ligature expansion (`ﬁ` → `fi`), soft-hyphen removal. *This is precisely the residual-error class RenderCV's study identified — normalising it is the difference between 96% and 100%.*
4. Assert every assertable IR field — name, contacts, organisations, roles, dates, skills, highlight prefixes — is present and in plausible reading order.
5. Structural checks via `lopdf`: fonts embedded, document title set, no text rendered as paths.

```console
$ cac check --strict
  ✓ 64/64 fields recovered from the PDF text layer
  ✓ fonts embedded · reading order OK
  PASS
```

This converts an unfalsifiable marketing claim into a CI test. Lead the README with it.

---

## 10. CLI

Six commands.

```
cac init [--from resume.json]    # creates cv.md
cac build [INPUT] [-t THEME] [-f pdf,html,json] [-o DIR]
          [--select EXPR] [--target NAME] [--max-pages N --fit]
cac watch [INPUT]
cac check [INPUT|PDF] [--fast] [--only CODES] [--strict]
          [--format human|json|sarif]
cac convert <IN> --to <yaml|json|toml|jsonresume>
cac themes [list|show|new]
```

Global: `-v/-q`, `--color`, `--json`. Machine-readable output on every command — non-negotiable for a tool that wants to live in CI.

Config (`cac.toml`) is **optional** and is the six lines shown in §7.

---

## 11. WASM

Two targets, different constraints.

| Target | Use |
|---|---|
| `wasm32-unknown-unknown` | Browser. The hard one: size and fonts. |
| `wasm32-wasip1` | Edge, CI sandboxes. Nearly free. |

- Typst compiles to WASM — `typst.ts` proves it in production. Study it; don't necessarily depend on it.
- **Don't embed fonts in the `.wasm`.** Load lazily over HTTP. 2 MB Latin default, packs on demand.
- Budget **< 8 MB gzipped**. Levers: `opt-level="z"`, fat LTO, `codegen-units=1`, `panic="abort"`, `wasm-opt -Oz`.
- Preview via **SVG** (`typst-svg`), not PDF — instant, zoomable, no PDF.js.
- Gate anything touching fs, process, or `SystemTime` behind `#[cfg(not(target_arch = "wasm32"))]`.

The playground web app is post-1.0.

---

## 12. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| **Typst pre-1.0 breaks each minor release** | High | Pin exact versions; confine all Typst API contact to `cac-render`; CI job tracking `typst@latest` so upgrades are known, not discovered |
| **Parity debt underestimated.** RenderCV has 9 themes, 22 locales, watch, 4 export formats, editor schema, agent skill. Reaching parity is most of the work | High | Don't let the fun features (fit, ATS check) crowd out the boring ones users notice missing on day one |
| **Theme quality is design work, not engineering** | High | Budget real design time. Port proven layouts rather than inventing. Two excellent themes beat nine adequate ones |
| Binary size (Typst + fonts) | Medium | `opt-level="z"`, LTO, strip, downloadable font packs |
| WASM bundle too large | Medium | Measured at M0 so it's a design input, not a week-20 surprise |
| Font licensing | Medium | OFL/Apache only — Libertinus, Source Sans 3, Noto, DejaVu, IBM Plex |
| **`cac` name unavailable** | Medium | Verify crates.io / Homebrew / winget / trademark **in week 1**. Homebrew-core has a notability bar for short names. Hold a fallback (`cacv`, `cvac`); register the crate immediately |
| Scope creep toward "AI resume builder" | High | The deterministic core is the product |

---

## 13. Testing

| Layer | Approach |
|---|---|
| Golden corpus | ~20 fixture CVs: minimal, maximal, Unicode-heavy, Vietnamese diacritics, 15-year career, and adversarial content (`C#`, `100%`, `~/`, `\`, emoji) in Markdown and structured formats |
| Snapshot | `insta` on IR JSON, HTML, plaintext |
| Round-trip property | `proptest`: arbitrary `CvDocument` → each format → reparse → assert equality. Catches adapter lossiness automatically |
| **ATS self-test** | `check --only CAC5xx --strict` over the corpus × both themes, in CI. The RenderCV study, automated, on every commit |
| Visual regression | Corpus → PNG → perceptual hash vs baselines. Maintainer tooling |
| CLI | `assert_cmd` / `trycmd` for exit codes and `--json` stability |
| Cross-platform | All six native targets. Path handling and font discovery are the usual failures |
| WASM | `wasm-bindgen-test` headless; **assert bundle size in CI**, fail on regression |

**Performance budgets** (fail CI on regression):

| Operation | Budget |
|---|---|
| Parse + validate | < 5 ms |
| Cold PDF render | < 250 ms |
| Warm incremental (`watch`) | < 60 ms |
| `--fit` solve | < 700 ms |
| `check --fast` | < 8 ms |
| `check` all phases | < 400 ms |
| Binary size, stripped | < 45 MB |
| WASM, gzipped | < 8 MB |

---

## 14. Distribution

**Targets:** `aarch64`/`x86_64-apple-darwin` (universal via `lipo`, codesigned + notarised), `x86_64`/`aarch64-unknown-linux-gnu` + `-musl`, `x86_64`/`aarch64-pc-windows-msvc`, `wasm32-unknown-unknown`, `wasm32-wasip1`.

**Toolchain:** `cargo-dist` (the `dist` CLI, actively maintained — v0.32 as of May 2026) generates the release workflow, builds the matrix, and produces source and binary archives plus shell and PowerShell installers. A separate workflow prepares the source-building Homebrew Core formula. `release-plz` handles changelog and version automation.

**Channels:** GitHub Releases + `curl | sh` (free from `dist`), Homebrew Core, winget via `wingetcreate`, Scoop bucket, crates.io, AUR, `cargo-deb`/`cargo-generate-rpm` artefacts, distroless Docker, and an **npm wrapper** — surprisingly effective distribution for a dev tool.

**Supply chain:** GitHub Artifact Attestations, SBOM via `cargo-cyclonedx`, `cargo-deny` + `cargo-audit` in CI, pinned toolchain in `rust-toolchain.toml`.

---

## 15. Roadmap

| | Scope | Effort |
|---|---|---|
| **M0** | Spike: embed Typst, implement `World`, prove the virtual `/cv.json` pattern, measure cold/warm compile + wasm size. **Gate:** if warm compile > 150 ms or wasm > 15 MB gzipped, revisit now | 3 days |
| **M1** | `cac-core` IR + Markdown-first `cac-io` + `cac-render` PDF + `init`/`build`, one theme, JSON Resume adapter. **First usable release** | 2 weeks |
| **M2** | Theme #2, HTML output, `watch`, `convert`, schema auto-reference | 1 week |
| **M3** | `--select`, `--fit`, optional `cac.toml` targets | 1 week |
| **M4** | `cac check`: 12 rules, 2 phases, ATS verification, SARIF | 1 week |
| **M5** | `cargo-dist` matrix, brew/winget/scoop/cargo/Docker/npm, docs, completions | 1 week |

**M1 is shippable alone.** Get `cac init && cac build` working on three platforms with zero dependencies and put it in front of people before building M3 or M4. The differentiators are worth building — just not before you know the core install experience holds up.

---

## 16. Open questions

1. **Photo support.** Legally discouraged in US/UK/DE hiring, standard across much of Asia and continental Europe. Support it and warn, or omit from v1?
2. **Section ordering** — source file order, or theme's choice? Proposal: source order wins; a theme may only *reject* an order it can't render.
3. **Closed `EntryKind` set.** Guarantees every theme renders every CV, at the cost of flexibility. `Text` is the escape hatch. Revisit with real user data.
4. **Is `--fit` worth M3 at all**, or does the check-phase page-budget warning (CAC402) cover 80% of the need for 10% of the work? Worth deciding after M2 with a real corpus.
