use clap::{Args, CommandFactory, Subcommand, ValueHint};
use clap::{Command, Parser};
use clap_complete::{Generator, Shell, generate};
use colored::*; // For coloring the output
use enginelib::prelude::error;
use enginelib::task::Task;
use enginelib::{api::EngineAPI, event::info};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use toml::Value;

#[derive(Debug)]
struct Entry {
    namespace: String,
    id: String,
    data: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct RawDoc(
    std::collections::BTreeMap<String, Vec<std::collections::BTreeMap<String, toml::Value>>>,
);

fn parse_entries(raw: RawDoc) -> Vec<Entry> {
    let mut result = Vec::new();

    for (compound_key, records) in raw.0 {
        // split on colon: "widget:button" -> ("widget", "button")
        let mut parts = compound_key.splitn(2, ':');
        let namespace = parts.next().unwrap_or("").to_string();
        let id = parts.next().unwrap_or("").to_string();

        for data in records {
            result.push(Entry {
                namespace: namespace.clone(),
                id: id.clone(),
                data,
            });
        }
    }

    result
}

/// A simple CLI application
#[derive(Parser, Debug)]
#[command(name = "packer")]
#[command(version = "1.0")]
#[command(author = "GrandEngineering")]
#[command(about = "A simple CLI app to pack tasks")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Commands {
    #[command()]
    Pack(PackArgs),
}
#[derive(Args, Debug, PartialEq)]
struct PackArgs {
    #[arg(short,required=true,value_hint=ValueHint::FilePath)]
    input: PathBuf,
}
fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        eprintln!("Generating completion file for {generator:?}...");
        print_completions(generator, &mut cmd);
    }
    let mut api = EngineAPI::default();
    EngineAPI::init_packer(&mut api);
    if let Some(command) = cli.command {
        match command {
            Commands::Pack(input) => {
                if input.input.exists() {
                    info!("Parsing File: {}", input.input.to_string_lossy());
                    let toml_str = std::fs::read_to_string(input.input).unwrap();
                    let raw: RawDoc = toml::from_str(&toml_str).unwrap();
                    let entries = parse_entries(raw);
                    for entry in entries {
                        println!("[{}:{}] => {:?}", entry.namespace, entry.id, entry.data);
                    }
                } else {
                    error!("File does not exist: {}", input.input.to_string_lossy())
                }
            }
        }
    }
}
