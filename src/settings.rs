/**
 * This file is part of the copydeps program.
 * Copyright (C) 2020 Artur "suve" Iwicki
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License,
 * either version 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program (LICENCE.txt). If not, see <https://www.gnu.org/licenses/>.
 */
use std::env;
use std::path::PathBuf;
use std::process::exit;
use std::vec::Vec;

extern crate getopts;
use getopts::Options;
use getopts::ParsingStyle;

use crate::version::*;

fn print_help() {
	print!(concat!(
			"{NAME} v.{VERSION} by {AUTHOR}\n",
			"Find and copy all .so / .dll files needed by a program to run.\n",
			"This can be useful when you want to bundle an application\n",
			"together will all its dependencies.\n",
			"\n",
			"Usage: {NAME} [options...] EXECUTABLE [TARGET-DIR]\n",
			"\n",
			"EXECUTABLE can be one of the following supported formats:\n",
			"- 32-bit ELF\n",
			"- 64-bit ELF\n",
			"- i386 Microsoft Windows executable\n",
			"- x86_64 Microsoft Windows executable\n",
			"\n",
			"TARGET-DIR specifies the directory to copy the .so / .dll files to.\n",
			"When omitted, defaults to the directory of the target executable.\n",
			"\n",
			"Program options:\n",
			"--dry-run\n",
			"  Print the list of dependencies without actually copying the .so / .dll files.\n",
			"--exedir\n",
			"  Include the directory of the executable in the .so / .dll resolve paths.\n",
			"  Files found in the exedir are preferred over those found anywhere else.\n",
			"--ignore PATTERN\n",
			"  Add PATTERN to the built-in ignore-list (.so / .dll names that should not\n",
			"  be resolved nor copied over).\n",
			"--no-clobber\n",
			"  Do not overwrite .so / .dll files already existing in the target directory.\n",
			"--override PATTERN\n",
			"  Add PATTERN to the override-list (.so / .dll names that should always be\n",
			"  resolved and copied over). Overrides have precedence over ignores.\n",
			"--search-dir DIRECTORY\n",
			"  Add DIRECTORY to the list of paths to search when resolving .so / .dll names.\n",
			"  User-specified directories take precedence over system paths.\n",
			"--verbose\n",
			"  Print the names of the dependencies as they're being copied over.\n",
			""
		),
		AUTHOR = PROGRAM_AUTHOR,
		NAME = PROGRAM_NAME,
		VERSION = PROGRAM_VERSION,
	);
}

fn verify_dir(dir: &PathBuf) -> Option<String> {
	if !dir.exists() {
		return Option::Some(format!("Directory \"{}\" does not exist", dir.to_str().unwrap()));
	}
	if !dir.is_dir() {
		return Option::Some(format!("\"{}\" is not a directory", dir.to_str().unwrap()));
	}
	return Option::None
}


pub struct Settings {
	pub dry_run: bool,
	pub executable: String,
	pub ignore_list: Vec<String>,
	pub no_clobber: bool,
	pub override_list: Vec<String>,
	pub search_dirs: Vec<String>,
	pub target_dir: String,
	pub verbose: bool,
}

impl Settings {
	pub fn new() -> Settings {
		return Settings {
			dry_run: false,
			executable: String::from(""),
			ignore_list: vec![],
			no_clobber: false,
			override_list: vec![],
			search_dirs: vec![],
			target_dir: String::from(""),
			verbose: false,
		};
	}

	pub fn new_from_argv() -> Result<Settings, String> {
		let args: Vec<String> = env::args().collect();
		let mut settings = Settings::new();

		let mut opts = Options::new();
		opts.parsing_style(ParsingStyle::FloatingFrees);
		opts.long_only(true);

		opts.optflag("", "help", "");
		opts.optflag("", "version", "");

		opts.optmulti("", "ignore", "", "");
		opts.optmulti("", "override", "", "");

		// Deprecated names for --ignore and --override. Present for backwards-compatibility.
		opts.optmulti("", "blacklist", "", "");
		opts.optmulti("", "whitelist", "", "");

		opts.optmulti("", "search-dir", "", "");

		opts.optflag("", "dry-run", "");
		opts.optflag("", "exedir", "");

		opts.optflag("", "no-clobber", "");
		opts.optflag("", "verbose", "");


		let matches = match opts.parse(args) {
			Ok(m) => { m },
			Err(f) => { return Result::Err(f.to_string()) }
		};

		if matches.opt_present("help") {
			print_help();
			exit(0);
		}
		if matches.opt_present("version") {
			println!("{} v.{} by {}", PROGRAM_NAME, PROGRAM_VERSION, PROGRAM_AUTHOR);
			exit(0);
		}

		match matches.free.len() {
			0 => return Result::Err(String::from("Failed to parse arguments")),
			1 => return Result::Err(String::from("Missing required argument: EXECUTABLE")),
			2 | 3 => {},
			_ => return Result::Err(String::from("Unexpected extra arguments"))
		}

		let mut executable = PathBuf::from(matches.free.get(1).unwrap());
		if !executable.exists() {
			return Result::Err(format!("File \"{}\" does not exist", executable.to_str().unwrap()));
		}
		if !executable.is_file() {
			return Result::Err(format!("File \"{}\" is not a regular file", executable.to_str().unwrap()));
		}

		executable = executable.canonicalize().unwrap();
		let executable_dir = executable.parent().unwrap();

		settings.executable = String::from(executable.to_str().unwrap());

		if matches.free.len() == 3 {
			let target_dir = PathBuf::from(matches.free.get(2).unwrap());
			match verify_dir(&target_dir) {
				Some(msg) => return Result::Err(msg),
				None => {},
			}
			settings.target_dir = String::from(target_dir.canonicalize().unwrap().to_str().unwrap());
		} else {
			settings.target_dir = String::from(executable_dir.to_str().unwrap());
		}

		settings.ignore_list = matches.opt_strs("ignore");
		settings.ignore_list.append(matches.opt_strs("blacklist").as_mut());

		settings.override_list = matches.opt_strs( "override");
		settings.override_list.append(matches.opt_strs("whitelist").as_mut());

		settings.search_dirs = matches.opt_strs("search-dir");

		if matches.opt_present("dry-run") {
			settings.dry_run = true;
		}
		if matches.opt_present("exedir") {
			settings.search_dirs.push(String::from(executable_dir.to_str().unwrap()));
		}
		if matches.opt_present("no-clobber") {
			settings.no_clobber = true;
		}
		if matches.opt_present("verbose") {
			settings.verbose = true;
		}

		return Result::Ok(settings);
	}
}
