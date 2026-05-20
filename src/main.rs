#![expect(clippy::as_conversions)]
#![expect(unused)]
#![allow(clippy::missing_const_for_fn)]
use std::env::args;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use clap::Parser;
use color_eyre::Section;
use color_eyre::eyre::{self, Result};
use colored::{ColoredString, Colorize};
use command_run::{Command, Error, Output};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use spinners::{Spinner, Spinners};

use crate::eyre::eyre;

static ERRFILE: &str = ".checkpoint.error";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Filename extension to watch (eg rs, js, py, java)
    #[arg(short, long, value_name = "filetype")]
    filetype: OsString,
    /// Command to run (use after -- if your shell requires it)
    command: Vec<OsString>,
    /// Don't run git commit when tests pass
    #[arg(short, long)]
    dryrun: bool,
    /// Clear screen between runs
    #[arg(short, long)]
    clear: bool,
    /// Don't display test output
    #[arg(short, long)]
    quiet: bool,
}

/// State diagram:
/// ```mermaid
/// flowchart LR
/// PASSING-->|fail|FAILING
/// FAILING-->|pass; git commit|PASSING
/// ```
/// Other transitions are no-ops (such as tests passing while in passing state)
#[derive(Debug, Copy, Clone)]
struct SavePoint<'a> {
    program: &'a Path,
    args: &'a [OsString],
    state: State,
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    Passing,
    Failing,
}
#[allow(clippy::enum_glob_use)]
use State::*;

//TODO: All flags should get saved into self in new()
impl<'a> SavePoint<'a> {
    /// If error file exists, failing, if not, passing
    fn new(program: &'a Path, args: &'a [OsString]) -> Self {
        let state = match fs::exists(ERRFILE) {
            Ok(_) => Passing,
            Err(_) => Failing,
        };
        Self {
            program,
            args,
            state,
        }
    }

    /// main state dispatcher
    fn test(mut self, program: &Path, dryrun: bool, quiet: bool) -> Result<Self> {
        let res = if quiet {
            let mut sp = Spinner::new(
                Spinners::Line,
                format!("Running {program}...", program = program.display()),
            );
            let res = cmdr(self.program, self.args, quiet);
            sp.stop();
            res
        } else {
            cmdr(self.program, self.args, quiet)
        };
        println!("done!");
        match (&self, res) {
            // noop
            (Self { state: Passing, .. }, Ok(_)) => Ok(self),
            (
                Self {
                    state: Failing | Passing,
                    ..
                },
                Err(_),
            ) => Ok(self.fail()),
            // notify, git commit
            (Self { state: Failing, .. }, Ok(_)) => self.pass(dryrun),
        }
    }

    /// fixed all errors, git commit
    fn pass(self, dryrun: bool) -> Result<Self> {
        commit("SAVEPOINT REACHED!", dryrun)?;
        rm_errfile()?;
        Ok(Self {
            state: Passing,
            ..self
        })
    }

    /// test just failed
    fn fail(self) -> Self {
        log(&"Error!".red().bold());
        let _ = create_errfile();
        Self {
            state: Failing,
            ..self
        }
    }
}

/// Clear ansi terminal and put cursor at top-left
fn clear() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn log(message: &ColoredString) {
    let prefix = "🏁 CHECKPOINT: ".blue().bold();
    print!("{prefix}");
    println!("{message}");
}

#[expect(clippy::result_large_err)]
fn cmdr(program: &Path, args: &[OsString], quiet: bool) -> Result<Output, Error> {
    let mut command = Command::with_args(program, args);
    if quiet {
        let command = command.enable_capture();
        command.combine_output = true;
    }
    command.log_command = false;
    command.run()
}
#[allow(clippy::panic_in_result_fn)]
#[allow(clippy::panic)]
fn main() -> Result<()> {
    // INFO: Setup
    color_eyre::install()?;
    let cli = Cli::parse();
    let dryrun = cli.dryrun;
    let quiet = cli.quiet;
    let extension = cli.filetype;
    let program = cli
        .command
        .first()
        .ok_or_else(|| eyre!("Missing argument: COMMAND"))?;
    let program = Path::new(program);
    let args = cli
        .command
        .get(1..)
        .ok_or_else(|| eyre!("no program arg"))?;

    //INFO: Ensure that if dryrun is not active, that the current environment
    // includes the git command
    if !dryrun {
        // We check that git exists by running git --version
        let mut git_version_command = Command::with_args("git", ["--version"]);
        git_version_command.log_command = false;
        match git_version_command.enable_capture().run() {
            Ok(_) => {}
            Err(e) => {
                if let command_run::ErrorKind::Run(run_error) = &e.kind
                    && run_error.kind() == std::io::ErrorKind::NotFound
                {
                    // git was not found
                    return Err(eyre!("could not find `git` command"));
                }
                // Another error occured
                return Err(eyre!(
                    "checking for `git` command failed with unexpected error {}",
                    e
                ));
            }
        }
    }

    //INFO: File Watcher
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    let mut machine = SavePoint::new(program, args);
    //INFO: Main UI Loop
    loop {
        log(&"Monitoring...".white().bold());
        machine = machine.test(program, dryrun, quiet)?;
        blockforfile(&rx, &extension);
        if cli.clear {
            clear();
        }
    }
}
fn blockforfile(rx: &Receiver<Result<Event, notify::Error>>, extension: &OsStr) {
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(Ok(Event {
                kind: EventKind::Modify(_),
                paths,
                ..
            })) if paths.first().map(|p| p.extension()) == Some(Some(extension)) => {
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

fn commit(msg: &str, dryrun: bool) -> Result<()> {
    if dryrun {
        log(&"(dry run) Autosaving!".green().bold());
        return Ok(());
    }
    log(&"Autosaving!".green().bold());
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

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(State::Passing, "which", "which")]
    #[case(State::Failing, "which", "nonexistingbin12345")]
    #[timeout(Duration::from_secs(1))]
    // TODO: Refactor this
    fn app_test(#[case] state: State, #[case] program: &str, #[case] params: &str) {
        let program = Path::new(program);
        let params = &[OsString::from(params)];
        let app = SavePoint::new(program, params);
        let run = app.test(program, true, true);
        assert_eq!(run.unwrap().state, state);
    }
}
