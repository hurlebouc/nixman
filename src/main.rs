use std::{fs::File, io::Write};

use askama::Template;
use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
// #[command(name = "git")]
#[command(about = "A Nix Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ///Initialize Nix repository
    Init,
}
#[derive(Template)]
#[template(path = "build.nix.j2")]
struct Build {}

#[derive(Template)]
#[template(path = "default.nix.j2")]
struct Default {}

#[derive(Template)]
#[template(path = "shell.nix.j2")]
struct Shell {}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Init => {
            let mut build_nix = File::create("build.nix")?;
            let mut shell_nix = File::create("shell.nix")?;
            let mut default_nix = File::create("default.nix")?;
            build_nix.write_all(Build {}.render().unwrap().as_bytes())?;
            shell_nix.write_all(Default {}.render().unwrap().as_bytes())?;
            default_nix.write_all(Shell {}.render().unwrap().as_bytes())?;
        }
    }
    Ok(())
}
