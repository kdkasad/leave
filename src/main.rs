//
// Copyright (C) 2025 Kian Kasad <kian@kasad.com>
//
// This file is part of Leave.
//
// Leave is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// Leave is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// Leave. If not, see <https://www.gnu.org/licenses/>.
//

#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    io::Error as IoError,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use eyre::{Context, bail};

#[derive(Debug, Parser)]
#[command(about, author, version)]
struct CliOptions {
    /// Files to leave present
    files: Vec<PathBuf>,

    /// Run as if started in <DIR>
    #[arg(long, short = 'C', value_name = "DIR")]
    chdir: Option<PathBuf>,

    /// Recursively delete directories and their contents
    #[arg(long, short)]
    recursive: bool,

    /// Delete empty directories
    #[arg(long, short)]
    dirs: bool,

    /// Continue even if some files given on the command line don't exist
    #[arg(long, short)]
    force: bool,
}

const MISTAKE_MSG: &str = "This is likely a mistake. To continue anyways, use -f/--force.";

fn main() -> ExitCode {
    match main_fallible() {
        Ok(code) => code,
        Err(err) => {
            print_error(&err);
            ExitCode::FAILURE
        }
    }
}

/// Wraps the actual error-prone logic so we can conveniently use `?` after
/// errors.
/// Returns `Ok(true)` if at least one error occurred while removing files, or
/// `Ok(false)` if successful.
fn main_fallible() -> eyre::Result<ExitCode> {
    let cli = CliOptions::parse();

    // Change directory to dir
    if let Some(dir) = &cli.chdir {
        std::env::set_current_dir(dir)
            .wrap_err_with(|| format!("Can't chdir into {}", dir.display()))?;
    }

    // Check arguments given to make sure they exist. If a user runs `leave
    // file.txt` but `file.txt` doesn't exist, it's probably a typo and we
    // shouldn't delete anything. The `-f, --force` flag overrides this.
    if !cli.force {
        if cli.files.is_empty() {
            bail!("No files provided. {MISTAKE_MSG}");
        }

        let mut abort = false;
        for arg in &cli.files {
            let exists = arg
                .try_exists()
                .wrap_err_with(|| format!("Can't check if {} exists", arg.display()))?;
            if !exists {
                eprintln!("Warning: {} doesn't exist.", arg.display());
                abort = true;
            }
        }
        if abort {
            bail!("One or more provided files don't exist. {MISTAKE_MSG}");
        }
    }

    // Get absolute paths to all arguments
    let cwd_absolute =
        std::path::absolute(".").wrap_err("Can't get path to current working directory")?;
    let absolute_files: HashSet<PathBuf> = cli
        .files
        .iter()
        .map(|p| -> eyre::Result<PathBuf> {
            let abs_path = std::path::absolute(p).wrap_err_with(|| format!("Can't make {} absolute", p.display()))?;
            if abs_path.parent().is_some_and(|parent| *parent != cwd_absolute) {
                bail!("{} is not in the current directory; it would be removed anyways. {MISTAKE_MSG}", p.display())
            }
            Ok(abs_path)
        })
        .collect::<Result<_, _>>()?;

    // Do removal
    let cwd = fs::read_dir(".").wrap_err("Can't list contents of .")?;
    let mut had_failure = false;
    for entry_result in cwd {
        if let Err(err) = process_entry(&cli, &absolute_files, entry_result) {
            // If an error occurs, print it but don't abort
            had_failure = true;
            print_error(&err);
        }
    }

    Ok(if had_failure {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn process_entry(
    cli: &CliOptions,
    absolute_files: &HashSet<PathBuf>,
    entry_result: Result<DirEntry, IoError>,
) -> eyre::Result<()> {
    let entry = entry_result.wrap_err("Can't read directory entry")?;
    let path = entry.path();
    let print_path = path.display();

    // Skip if matches one of the arguments
    let entry_absolute = std::path::absolute(entry.path())
        .wrap_err_with(|| format!("Can't make {print_path} absolute"))?;
    if absolute_files.contains(&entry_absolute) {
        return Ok(());
    }

    let file_type = entry
        .file_type()
        .wrap_err_with(|| format!("Can't get type of {print_path}"))?;
    let result: eyre::Result<()> = if file_type.is_dir() {
        delete_dir(cli, &entry.path())
    } else {
        fs::remove_file(entry.path()).map_err(eyre::Report::from)
    };
    result.wrap_err_with(|| format!("Can't remove {print_path}"))
}

/// Deletes a directory according to the CLI options given.
fn delete_dir(cli: &CliOptions, dir: &Path) -> eyre::Result<()> {
    if cli.recursive {
        // If recursive directory deletion is enabled, we can delete all directories
        fs::remove_dir_all(dir)?;
    } else if !cli.dirs {
        // If recursive and empty directory deletion are disabled, we can't delete any directories
        bail!("Is a directory");
    } else {
        // We can delete empty directories only

        // Check if directory is empty
        let mut dir_iter = dir
            .read_dir()
            .wrap_err_with(|| format!("Can't list contents of {}", dir.display()))?;
        let is_empty = dir_iter.next().is_none();

        if is_empty {
            fs::remove_dir(dir)?;
        } else {
            bail!("Directory is not empty");
        }
    }

    Ok(())
}

/// Prints the given error to standard error.
///
/// Prints the full cause chain in a single line, separated by colons.
fn print_error(error: &eyre::Report) {
    eprint!("Error: ");
    for (i, err) in error.chain().enumerate() {
        let prefix = if i > 0 { ": " } else { "" };
        eprint!("{prefix}{err}");
    }
    eprintln!();
}
