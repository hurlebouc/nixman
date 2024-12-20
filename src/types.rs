use clap::Subcommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package(pub String);

impl Package {
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Language {
    Rust,
    Go,
    Maven,
}
