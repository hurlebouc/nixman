mod types;

use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write},
    process::Command,
};

use askama::Template;
use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
// #[command(name = "git")]
#[command(about = "A Nix Manager", long_about = None, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ///Initialize Nix repository
    Init {
        /// Optional channel
        #[arg(short, long, default_value = "nixos-unstable")]
        channel: String,

        /// Optional language
        #[command(subcommand)]
        language: Option<types::Language>,
    },
    Code {
        path: String,
    },
}
#[derive(Template)]
#[template(path = "build.nix.j2")]
struct Build {}

#[derive(Template)]
#[template(path = "default.nix.j2")]
struct Default {
    channel: String,
    packages: Vec<types::Package>,
    shell_attrs: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "shell.nix.j2")]
struct Shell {}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Init { channel, language } => {
            let mut build_nix = File::create("build.nix")?;
            let mut shell_nix = File::create("shell.nix")?;
            let mut default_nix = File::create("default.nix")?;
            build_nix.write_all(Build {}.render().unwrap().as_bytes())?;
            shell_nix.write_all(Shell {}.render().unwrap().as_bytes())?;
            default_nix.write_all(
                Default {
                    channel,
                    packages: {
                        let mut packages = match language {
                            Some(types::Language::Rust) => vec![],
                            None => vec![],
                        };
                        packages.push(types::Package("pkgs.nixpkgs-fmt".to_string()));
                        packages
                    },
                    shell_attrs: match language {
                        Some(_) => {
                            let mut attrs = HashMap::new();
                            attrs.insert(
                                "RUST_SRC_PATH".to_string(),
                                "\"${pkgs.rustPlatform.rustLibSrc}\"".to_string(),
                            );
                            attrs
                        }
                        None => HashMap::new(),
                    },
                }
                .render()
                .unwrap()
                .as_bytes(),
            )?;
        }
        Commands::Code { path } => {
            let exit_status = Command::new("nix-shell")
                .arg(format!("{}/shell.nix", path))
                .arg("--run")
                .arg(format!("code {}", path))
                .status()?;
            if !exit_status.success() {
                return Err(io::Error::other("VSCode launched failed"));
            }
        }
    }
    Ok(())
}
