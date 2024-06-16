mod types;

use std::{
    collections::HashMap,
    ffi::OsString,
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
#[template(path = "build_rust.nix.j2")]
struct BuildRust {}

#[derive(Template)]
#[template(path = "gitignore.j2")]
struct Gitignore {
    ignores: Vec<String>,
}

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
            println!("Init nix...");
            let mut build_nix = File::create("build.nix")?;
            let mut shell_nix = File::create("shell.nix")?;
            let mut default_nix = File::create("default.nix")?;
            let mut gitignore = File::create(".gitignore")?;
            build_nix.write_all(
                match language {
                    Some(types::Language::Rust) => BuildRust {}.render().unwrap(),
                    None => Build {}.render().unwrap(),
                }
                .as_bytes(),
            )?;
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
            gitignore.write_all(
                Gitignore {
                    ignores: match language {
                        Some(types::Language::Rust) => vec!["/target".to_string()],
                        None => vec![],
                    },
                }
                .render()
                .unwrap()
                .as_bytes(),
            )?;

            let mut cmd_str = String::new();
            cmd_str.push_str("echo 'git init' && git init");
            cmd_str.push_str("&& echo 'git add *.nix' && git add *.nix");
            if let Some(types::Language::Rust) = language {
                cmd_str.push_str("&& echo 'cargo init' && cargo init --bin");
                cmd_str.push_str("&& echo 'cargo build' && cargo build");
                cmd_str.push_str("&& echo 'git add rust' && git add src Cargo.toml Cargo.lock");
            }
            cmd_str.push_str("&& echo 'git add .gitignore' && git add .gitignore");
            cmd_str.push_str(&format!(
                "&& echo 'git commit' && git commit -m \"{}\"",
                std::env::args().collect::<Vec<String>>().join(" ")
            ));

            println!("Entering nix...");
            let mut cmd = Command::new("nix-shell");
            cmd.arg("--run").arg(cmd_str);
            cmd.status()?;
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
