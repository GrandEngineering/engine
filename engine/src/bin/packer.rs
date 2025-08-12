use clap::{Args, CommandFactory, Subcommand, ValueHint};
use clap::{Command, Parser};
use clap_complete::{Generator, Shell, generate};
use colored::*;
use enginelib::events::ID;
// For coloring the output
use enginelib::Registry;
use enginelib::prelude::error;
use enginelib::task::{StoredTask, Task, TaskQueue};
use enginelib::{api::EngineAPI, event::info};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};
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
    #[command()]
    Unpack(PackArgs),
    #[command()]
    Schema,
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
    for (id, tsk) in api.task_registry.tasks.iter() {
        api.task_queue.tasks.entry(id.clone()).or_default();
    }
    if let Some(command) = cli.command {
        match command {
            Commands::Schema => {
                let mut buf: Vec<String> = Vec::new();
                for tsk in api.task_registry.tasks {
                    let unw = tsk.1.to_toml();
                    buf.push(format![r#"[["{}:{}"]]"#, tsk.0.0, tsk.0.1]);
                    buf.push(unw);
                }
                let ns = buf.join("\n");
                let mut file = File::create("schema.rustforge.toml").unwrap();
                file.write_all(ns.as_bytes()).unwrap();
            }
            Commands::Unpack(input) => {
                if input.input.exists() {
                    let mut final_out: Vec<String> = Vec::new();
                    info!("Unpacking File: {}", input.input.to_string_lossy());
                    let mut buf = Vec::new();
                    File::open(input.input)
                        .unwrap()
                        .read_to_end(&mut buf)
                        .unwrap();
                    let k: TaskQueue = bincode::deserialize(&buf).unwrap();
                    for tasks in k.tasks {
                        let tt = api.task_registry.tasks.get(&tasks.0.clone()).unwrap();
                        for task in tasks.1 {
                            if tt.verify(task.bytes.clone()) {
                                let tmp_nt = tt.from_bytes(&task.bytes);
                                final_out.push(format![
                                    r#"[["{}:{}"]]"#,
                                    tasks.0.0.clone(),
                                    tasks.0.1.clone()
                                ]);
                                final_out.push(tmp_nt.to_toml());
                                info!("{:?}", tmp_nt);
                            };
                        }
                    }
                    let ns = final_out.join("\n");
                    let mut file = File::create("output.rustforge.toml").unwrap();
                    file.write_all(ns.as_bytes()).unwrap();
                }
            }
            Commands::Pack(input) => {
                if input.input.exists() {
                    info!("Packing File: {}", input.input.to_string_lossy());
                    let toml_str = std::fs::read_to_string(input.input).unwrap();
                    let raw: RawDoc = toml::from_str(&toml_str).unwrap();
                    let entries = parse_entries(raw);
                    for entry in entries {
                        let template = api
                            .task_registry
                            .get(&ID(entry.namespace.as_str(), entry.id.as_str()))
                            .unwrap();
                        let toml_string = toml::to_string(&entry.data).unwrap();
                        let t = template.from_toml(toml_string);
                        let mut tmp = api
                            .task_queue
                            .tasks
                            .get(&ID(entry.namespace.as_str(), entry.id.as_str()))
                            .unwrap()
                            .clone();
                        tmp.push(StoredTask {
                            id: "".into(), //ids are minted on the server
                            bytes: t.to_bytes(),
                        });
                        api.task_queue
                            .tasks
                            .insert(ID(entry.namespace.as_str(), entry.id.as_str()), tmp);
                    }
                    let data = bincode::serialize(&api.task_queue).unwrap();
                    let mut file = File::create("output.rustforge.bin").unwrap();
                    file.write_all(&data).unwrap();
                } else {
                    error!("File does not exist: {}", input.input.to_string_lossy())
                }
            }
        }
    }
}
