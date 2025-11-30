leave — Inverted rm(1) command
===

`leave` is an inverted `rm` command. It removes everything in the current directory except the files given as arguments.

Example:
```
$ ls
main.rs other.rs test.rs
$ leave main.rs
$ ls
main.rs
```

# Install

Install from Git sources using Cargo:
```
$ cargo install --git https://github.com/kdkasad/leave
```

# Usage

```
$ leave --help
Usage: leave [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Files to leave present

Options:
  -C, --chdir <DIR>  Run as if started in <DIR>
  -r, --recursive    Recursively delete directories and their contents
  -d, --dirs         Delete empty directories
  -f, --force        Continue even if some files given on the command line don't exist
  -h, --help         Print help
  -V, --version      Print version
```

# License

Copyright (C) 2025 Kian Kasad ([@kdkasad])

Leave is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

Leave is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
Leave. If not, see <https://www.gnu.org/licenses/>.

[@kdkasad]: https://github.com/kdkasad
