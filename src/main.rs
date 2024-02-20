//! A minimal tool, which only lets you create a quick note and open it using your default text
//! editor. You will also be able to list the notes, find in them and delete them.
//!
//! It will be implemented in a way so it is easily usable with other unix tools such as `grep` and
//! `find`.
//!
//! Maybe will even be able to launch a local server to preview the notes

use std::{fs, process::Stdio, time::SystemTime};

use args::{Action, Args};
use clap::Parser;
use config::Config;
use tokio::process::Command;

mod args;
mod config;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::new(args.config);
    let editor = config
        .editor
        .unwrap_or_else(|| std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string()));

    if !config.notes_dir.exists() {
        fs::create_dir_all(&config.notes_dir).unwrap();
    }

    match args.action {
        Action::New { name } => {
            let name = name.unwrap_or_else(|| {
                let time_now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string();
                format!("{time_now}.md")
            });

            Command::new(&editor)
                .arg(config.notes_dir.join(name))
                .spawn()
                .expect("Failed to spawn editor")
                .wait()
                .await
                .expect("An error occured while running editor");
        }
        Action::List => {
            for entry in fs::read_dir(config.notes_dir).unwrap() {
                let dir = entry.unwrap();
                println!("{}", dir.file_name().to_string_lossy());
            }
        }
        Action::Find => {
            let child = Command::new("fzf")
                .kill_on_drop(true)
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to spawn fzf");

            let output = child.wait_with_output().await.unwrap();
            let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();

            println!("{}{}", config.notes_dir.display(), selected);
        }
        Action::View { name } => {
            println!(
                "{}",
                fs::read_to_string(config.notes_dir.join(name)).unwrap()
            );
        }
        Action::Remove { name } => fs::remove_file(config.notes_dir.join(name)).unwrap(),
        Action::Interactive => {
            todo!()
        }
    }
}
