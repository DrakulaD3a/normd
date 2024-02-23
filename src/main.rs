//! A minimal tool, which only lets you create a quick note and open it using your default text
//! editor. You will also be able to list the notes, find in them and delete them.
//!
//! It will be implemented in a way so it is easily usable with other unix tools such as `grep` and
//! `find`.
//!
//! Maybe will even be able to launch a local server to preview the notes

use std::{
    fs,
    io::stdin,
    process::Stdio,
    time::{Duration, SystemTime},
};

use args::{Action, Args};
use clap::Parser;
use config::Config;
use tokio::process::Command;

mod args;
mod config;
mod server;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::new(args.config);
    let editor = config
        .editor
        .unwrap_or_else(|| std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string()));

    if !config.notes_dir.exists() {
        fs::create_dir_all(&config.notes_dir).unwrap_or_else(|e| {
            eprintln!("Failed to create notes directory: {e}");
            // TODO: Exit codes
            std::process::exit(1);
        });
    }

    match args.action {
        Action::New { name } => {
            let mut name = name.unwrap_or_else(|| {
                let time_now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs()
                    .to_string();
                format!("{time_now}.md")
            });

            if !name.ends_with(".md") {
                name += ".md";
            }

            Command::new(&editor)
                .arg(config.notes_dir.join(name))
                .spawn()
                .unwrap_or_else(|e| {
                    eprintln!("Failed to spawn editor: {e}");
                    // TODO: Exit codes
                    std::process::exit(1);
                })
                .wait()
                .await
                .unwrap_or_else(|e| {
                    eprintln!("An error occured while running editor: {e}");
                    // TODO: Exit codes
                    std::process::exit(1);
                });
        }
        Action::List => {
            for entry in fs::read_dir(config.notes_dir).unwrap_or_else(|e| {
                eprintln!("Failed to read notes directory: {e}");
                // TODO: Exit codes
                std::process::exit(1);
            }) {
                let dir = entry.unwrap_or_else(|e| {
                    eprintln!("Failed to read notes directory: {e}");
                    // TODO: Exit codes
                    std::process::exit(1);
                });
                println!("{}", dir.file_name().to_string_lossy());
            }
        }
        Action::Find => {
            let child = Command::new("fzf")
                .kill_on_drop(true)
                .current_dir(&config.notes_dir)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap_or_else(|e| {
                    eprintln!("Failed to spawn fzf: {e}");
                    // TODO: Exit codes
                    std::process::exit(1);
                });

            let output = child.wait_with_output().await.unwrap_or_else(|e| {
                eprintln!("An error occured while running fzf!: {e}");
                // TODO: Exit codes
                std::process::exit(1);
            });
            let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();

            println!("{}{}", config.notes_dir.display(), selected);
        }
        Action::View { name } => {
            println!(
                "{}",
                match fs::read_to_string(config.notes_dir.join(get_name_or_stdin(name))) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Failed to read file: {e}");
                        // TODO: Exit codes
                        std::process::exit(1);
                    }
                }
            );
        }
        Action::Remove { name } => fs::remove_file(config.notes_dir.join(get_name_or_stdin(name)))
            .unwrap_or_else(|e| eprintln!("Failed to remove file: {e}")),
        Action::Interactive => {
            println!("Not yet implemented");
        }
        Action::Serve { port } => {
            server::Server::new(port.unwrap_or(8080), &config.notes_dir)
                .serve()
                .await
                .unwrap();
        }
    }
}

fn get_name_or_stdin(name: Option<String>) -> String {
    name.unwrap_or_else(|| {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap_or_else(|e| {
            eprintln!("Failed to read from stdin: {e}");
            // TODO: Exit codes
            std::process::exit(1);
        });
        buf.trim().to_string()
    })
}
