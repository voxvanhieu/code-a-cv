use crate::error::Result;

pub const ABOUT: &str = "List embedded themes";
pub const AFTER_HELP: &str = "Example:\n  cac themes";

pub fn run() -> Result<()> {
    println!("classic (embedded)");
    Ok(())
}
