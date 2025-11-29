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
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use eyre::{Context, bail};

#[derive(Debug, Parser)]
#[command(about)]
struct CliOptions {
    /// Files to leave present
    files: Vec<PathBuf>,

    /// Run as if started in <DIR>
    #[arg(long, short = 'C', value_name = "DIR")]
    chdir: Option<PathBuf>,

    /// If set, will recursively delete directories
    #[arg(long, short)]
    recursive: bool,

    /// If set, will delete empty directories
    #[arg(long, short)]
    dirs: bool,

    /// Continue even if some files given on the command line don't exist
    #[arg(long, short)]
    force: bool,
}

fn main() -> ExitCode {
    if let Err(report) = main_fallible() {
        eprint!("Error: ");
        for (i, err) in report.chain().enumerate() {
            let prefix = if i > 0 { ": " } else { "" };
            eprint!("{prefix}{err}");
        }
        eprintln!();
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Wraps the actual error-prone logic so we can conveniently use `?` after
/// errors
fn main_fallible() -> eyre::Result<()> {
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
            bail!(
                "One or more provided files don't exist. This is likely a mistake. To continue anyways, use -f/--force."
            );
        }
    }

    // Get absolute paths to all arguments
    let absolute_files: HashSet<PathBuf> = cli
        .files
        .iter()
        .map(|p| {
            std::path::absolute(p).wrap_err_with(|| format!("Can't make {} absolute", p.display()))
        })
        .collect::<Result<_, _>>()?;

    // Do removal
    let cwd = fs::read_dir(".").wrap_err("Can't list contents of .")?;
    for entry_result in cwd {
        let entry = entry_result.wrap_err("Can't read directory entry")?;
        let path = entry.path();
        let print_path = path.display();

        // Skip if matches one of the arguments
        let entry_absolute = std::path::absolute(entry.path())
            .wrap_err_with(|| format!("Can't make {print_path} absolute"))?;
        if absolute_files.contains(&entry_absolute) {
            continue;
        }

        let file_type = entry
            .file_type()
            .wrap_err_with(|| format!("Can't get type of {print_path}"))?;
        let result: eyre::Result<()> = if file_type.is_dir() {
            delete_dir(&cli, &entry.path())
        } else {
            fs::remove_file(entry.path()).map_err(eyre::Report::from)
        };
        result.wrap_err_with(|| format!("Can't remove {print_path}"))?;
    }

    Ok(())
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
