mod types;

use std::{
    collections::HashMap,
    env::current_dir,
    fs::File,
    io::{self, stdin, stdout, Write},
    process::Command,
};

use askama::Template;
use clap::{Parser, Subcommand};
use regex::Regex;
use types::Package;

/// A Nix Manager for developers
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
    Build,
    Install,
}
#[derive(Template)]
#[template(path = "build.nix.j2")]
struct Build {}
#[derive(Template)]
#[template(path = "build_rust.nix.j2")]
struct BuildRust {}

#[derive(Template)]
#[template(path = "build_go.nix.j2")]
struct BuildGo {
    name: String,
}

#[derive(Template)]
#[template(path = "main.go.j2")]
struct MainGo {}

#[derive(Template)]
#[template(path = "gitignore.j2")]
struct Gitignore {
    ignores: Vec<String>,
}

#[derive(Template)]
#[template(path = "default.nix.j2")]
struct Default {
    channel: String,
    shell_packages: Vec<types::Package>,
    shell_attrs: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "shell.nix.j2")]
struct Shell {}

fn check_word(s: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
    re.is_match(s)
}

fn check_path(s: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9.\/]+$").unwrap();
    re.is_match(s)
}

fn ask_line(question: &str) -> std::io::Result<String> {
    //print!("{}", question);
    stdout().write_all(question.as_bytes())?;
    stdout().flush()?;
    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    Ok(answer.trim().to_string())
}

fn ask_word(question: &str) -> std::io::Result<String> {
    let res = ask_line(question)?;
    if !check_word(&res) {
        Err(std::io::Error::other("Expect word input"))
    } else {
        Ok(res)
    }
}

fn ask_path(question: &str) -> std::io::Result<String> {
    let res = ask_line(question)?;
    if !check_path(&res) {
        Err(std::io::Error::other("Expect path input"))
    } else {
        Ok(res)
    }
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Init { channel, language } => {
            println!("Init nix...");
            let name = current_dir()?
                .file_name()
                .map(|os| os.to_string_lossy().to_string())
                .unwrap_or_else(|| ask_word("Name: ").unwrap());
            let mut build_nix = File::create("build.nix")?;
            let mut shell_nix = File::create("shell.nix")?;
            let mut default_nix = File::create("default.nix")?;
            let mut gitignore = File::create(".gitignore")?;
            build_nix.write_all(
                match language {
                    Some(types::Language::Rust) => BuildRust {}.render().unwrap(),
                    Some(types::Language::Go) => BuildGo { name: name.clone() }.render().unwrap(),
                    None => Build {}.render().unwrap(),
                }
                .as_bytes(),
            )?;
            shell_nix.write_all(Shell {}.render().unwrap().as_bytes())?;
            default_nix.write_all(
                Default {
                    channel,
                    shell_packages: {
                        let mut packages = match language {
                            Some(types::Language::Rust) => vec![Package("git".to_string())],
                            Some(types::Language::Go) => vec![Package("git".to_string())],
                            None => vec![],
                        };
                        packages.push(types::Package("nixpkgs-fmt".to_string()));
                        packages
                    },
                    shell_attrs: match language {
                        Some(types::Language::Rust) => {
                            let mut attrs = HashMap::new();
                            attrs.insert(
                                "RUST_SRC_PATH".to_string(),
                                "\"${pkgs.rustPlatform.rustLibSrc}\"".to_string(),
                            );
                            attrs
                        }
                        Some(types::Language::Go) => {
                            let mut attrs = HashMap::new();
                            attrs.insert(
                                "GOPATH".to_string(),
                                "\"${PROJECT_ROOT}/gohome/go\"".to_string(),
                            );
                            attrs.insert(
                                "GOCACHE".to_string(),
                                "\"${PROJECT_ROOT}/gohome/cache\"".to_string(),
                            );
                            attrs.insert(
                                "GOENV".to_string(),
                                "\"${PROJECT_ROOT}/gohome/env\"".to_string(),
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
                        Some(types::Language::Go) => vec!["/gohome".to_string()],
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
            match language {
                Some(types::Language::Rust) => {
                    cmd_str.push_str("&& echo 'cargo init' && cargo init --bin");
                    cmd_str.push_str("&& echo 'cargo build' && cargo build");
                    cmd_str.push_str("&& echo 'git add rust' && git add src Cargo.toml Cargo.lock");
                }
                Some(types::Language::Go) => {
                    let mut main_go = File::create("main.go")?;
                    main_go.write_all(MainGo {}.render().unwrap().as_bytes())?;
                    let path = ask_path(&format!("Access path (ex. github.com/plop/{}): ", &name))?;
                    cmd_str.push_str(&format!("&& echo 'go mod init' && go mod init {}", path));
                    cmd_str.push_str("&& echo 'go mod tidy' && go mod tidy");
                    cmd_str.push_str("&& echo 'git add go' && git add main.go go.mod");
                }
                None => {}
            }
            cmd_str.push_str("&& echo 'git add .gitignore' && git add .gitignore");
            cmd_str.push_str(&format!(
                "&& echo 'git commit' && git commit -m \"{}\"",
                std::env::args().collect::<Vec<String>>().join(" ")
            ));

            println!("Entering nix...");
            let mut cmd = Command::new("nix-shell");
            cmd.arg("--pure");
            cmd.arg("--run").arg(cmd_str);
            cmd.arg("-p");
            cmd.arg("git");
            match language {
                Some(types::Language::Rust) => {
                    cmd.arg("cargo");
                }
                Some(types::Language::Go) => {
                    cmd.arg("go");
                }
                None => {}
            }
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
        Commands::Build => {
            let exit_status = Command::new("nix-build").arg("-A").arg("build").status()?;
            if !exit_status.success() {
                return Err(io::Error::other("Error building project"));
            }
        }
        Commands::Install => {
            let exit_status = Command::new("nix-env")
                .arg("-f")
                .arg(".")
                .arg("--install")
                .arg("-A")
                .arg("build")
                .status()?;
            if !exit_status.success() {
                return Err(io::Error::other("Error installing project"));
            }
        }
    }
    Ok(())
}
