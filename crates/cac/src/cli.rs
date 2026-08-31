use cac_io::InputFormat;
use clap::{Parser, Subcommand, ValueEnum};

use crate::commands::{build, check, convert, init, themes};
use crate::error::Result;

#[derive(Parser)]
#[command(
    name = "cac",
    version,
    about = "Code a CV helps you manage your CV(s) as Code: version-controlled, tailored per job requirement, and consistently formatted.",
    after_help = "Homepage: https://github.com/voxvanhieu/code-a-cv\nReport bugs: https://github.com/voxvanhieu/code-a-cv/issues"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = init::ABOUT, after_help = init::AFTER_HELP)]
    Init(init::Args),
    #[command(about = build::ABOUT, after_help = build::AFTER_HELP)]
    Build(build::Args),
    #[command(about = check::ABOUT, after_help = check::AFTER_HELP)]
    Check(check::Args),
    #[command(about = convert::ABOUT, after_help = convert::AFTER_HELP)]
    Convert(convert::Args),
    #[command(about = themes::ABOUT, after_help = themes::AFTER_HELP)]
    Themes,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Command::Init(args) => init::run(args),
            Command::Build(args) => build::run(args),
            Command::Check(args) => check::run(args),
            Command::Convert(args) => convert::run(args),
            Command::Themes => themes::run(),
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CvFormat {
    Markdown,
    Yaml,
    Json,
    Toml,
    Jsonresume,
}

impl From<CvFormat> for InputFormat {
    fn from(value: CvFormat) -> Self {
        match value {
            CvFormat::Markdown => Self::Markdown,
            CvFormat::Yaml => Self::Yaml,
            CvFormat::Json => Self::Json,
            CvFormat::Toml => Self::Toml,
            CvFormat::Jsonresume => Self::JsonResume,
        }
    }
}
