#![expect(clippy::as_conversions)]

use crate::eyre::eyre;
use clap::Parser;
use color_eyre::Section;
use color_eyre::eyre::{self, Result};
use colored::{ColoredString, Colorize};
use command_run::Command;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::{ffi::OsStr, fs, sync::mpsc::Receiver, time::Duration};
use std::{path::Path, sync::mpsc};

static ERRFILE: &str = ".checkpoint.error";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Filename extension to watch
    #[arg(short, long, value_name = "filetype")]
    filetype: String,
    name: Vec<String>,
}

fn clear() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn log(message: &ColoredString) {
    let prefix = "🏁 CHECKPOINT: ".blue().bold();
    print!("{prefix}");
    println!("{message}");
}

fn main() -> Result<()> {
    // INFO: Setup
    color_eyre::install()?;
    let cli = Cli::parse();
    let extension = cli.filetype;
    let program = cli
        .name
        .first()
        .ok_or_else(|| eyre!("Missing argument: COMMAND"))?;
    let args = cli.name.get(1..).ok_or_else(|| eyre!("no program arg"))?;

    //INFO: File Watcher
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    //INFO: Main UI Loop
    loop {
        clear(); //TODO: whereshould this go?
        log(&"Running command...".white().bold());
        let mut command = Command::with_args(program, args);
        command.log_command = false;
        let output = command.run();
        if output.is_err() {
            //INFO: ERROR
            log(&"Error!".red().bold());
            create_errfile()?;
        } else {
            //INFO: NO ERROR
            if fs::exists(ERRFILE)? {
                log(&"Autosaving!".green().bold());
                #[allow(clippy::expect_used)]
                commit("CHECKPOINT SAVED!")?;
                rm_errfile()?;
            } else {
                log(&"OK".green().bold());
            }
        }
        log(&"Monitoring...".white().bold());
        blockforfile(&rx, &extension);
    }
}
fn blockforfile(rx: &Receiver<Result<Event, notify::Error>>, extension: &str) {
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(Ok(Event {
                kind: EventKind::Modify(_),
                paths,
                ..
            })) if paths.first().map(|p| p.extension()) == Some(Some(OsStr::new(extension))) => {
                break;
            }
            _ => {
                // ignoring
            }
        }
    }
    while rx.recv_timeout(Duration::from_millis(100)).is_ok() {
        // DRAIN THE CHANNEL
    }
}

fn commit(msg: &str) -> Result<()> {
    let mut command = Command::with_args("git", ["commit", "-am", msg]);
    command.log_command = false;
    if command.run().is_ok() {
        Ok(())
    } else {
        log(&"Fatal error!".red().bold());
        Err(eyre!("Git command error.")
            .with_suggestion(|| "Consider manually removing the `.checkpoint.error` file"))
    }
}

fn create_errfile() -> Result<()> {
    let mut command = Command::with_args("touch", [ERRFILE]);
    command.log_command = false;
    command.run()?;
    Ok(())
}
fn rm_errfile() -> Result<()> {
    let mut command = Command::with_args("rm", [ERRFILE]);
    command.log_command = false;
    command.run()?;
    Ok(())
}
