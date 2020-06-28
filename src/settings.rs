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

extern crate regex;
use regex::RegexSet;
use regex::RegexSetBuilder;

use crate::exit_status::*;
use crate::version::*;

fn print_help() {
	print!(concat!(
			"{NAME} finds and copies all .so / .dll files needed by a program to run.\n",
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
			"  Add PATTERN to the ignore-list (.so / .dll names that should not\n",
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
		NAME = PROGRAM_NAME,
	);
}

fn print_version() {
	println!("{} v.{} by {}", PROGRAM_NAME, PROGRAM_VERSION, PROGRAM_AUTHOR);
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
	pub executable: PathBuf,
	pub ignore_list: RegexSet,
	pub no_clobber: bool,
	pub override_list: RegexSet,
	pub search_dirs: Vec<PathBuf>,
	pub target_dir: PathBuf,
	pub verbose: bool,

	ignore_list_str: Vec<String>,
	override_list_str: Vec<String>,
}

impl Settings {
	pub fn new() -> Settings {
		let empty_vector: Vec<&str> = vec![];
		return Settings {
			dry_run: false,
			executable: PathBuf::new(),
			ignore_list: RegexSet::new(&empty_vector).unwrap(),
			no_clobber: false,
			override_list: RegexSet::new(&empty_vector).unwrap(),
			search_dirs: vec![],
			target_dir: PathBuf::new(),
			verbose: false,

			ignore_list_str: vec![],
			override_list_str: vec![],
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
			exit(EXIT_OK);
		}
		if matches.opt_present("version") {
			print_version();
			exit(EXIT_OK);
		}

		match matches.free.len() {
			0 => return Result::Err(String::from("Failed to parse arguments")),
			1 => return Result::Err(String::from("Missing required argument: EXECUTABLE")),
			2 | 3 => {},
			_ => return Result::Err(String::from("Unexpected extra arguments"))
		}

		let executable = PathBuf::from(matches.free.get(1).unwrap());
		if !executable.exists() {
			return Result::Err(format!("File \"{}\" does not exist", executable.to_str().unwrap()));
		}
		if !executable.is_file() {
			return Result::Err(format!("File \"{}\" is not a regular file", executable.to_str().unwrap()));
		}

		settings.executable = match executable.canonicalize() {
			Ok(pb) => pb,
			Err(msg) => return Result::Err(format!("Failed to canonicalize path \"{}\": {}", executable.to_string_lossy(), msg))
		};
		let executable_dir = settings.executable.parent().unwrap().to_path_buf();

		if matches.free.len() == 3 {
			let target_dir = PathBuf::from(matches.free.get(2).unwrap());
			match verify_dir(&target_dir) {
				Some(msg) => return Result::Err(msg),
				None => {},
			}
			settings.target_dir = match target_dir.canonicalize() {
				Ok(pb) => pb,
				Err(msg) => return Result::Err(format!("Failed to canonicalize path \"{}\": {}", target_dir.to_string_lossy(), msg))
			};
		} else {
			settings.target_dir = executable_dir.clone();
		}

		settings.ignore_list_str = matches.opt_strs("ignore");
		settings.ignore_list_str.append(matches.opt_strs("blacklist").as_mut());

		settings.override_list_str = matches.opt_strs( "override");
		settings.override_list_str.append(matches.opt_strs("whitelist").as_mut());

		for entry in matches.opt_strs("search-dir") {
			let entry_pb = PathBuf::from(entry);
			match verify_dir(&entry_pb) {
				Some(msg) => return Result::Err(msg),
				None => settings.search_dirs.push(entry_pb),
			}
		}

		if matches.opt_present("dry-run") {
			settings.dry_run = true;
		}
		if matches.opt_present("exedir") {
			settings.search_dirs.insert(0, executable_dir.clone());
		}
		if matches.opt_present("no-clobber") {
			settings.no_clobber = true;
		}
		if matches.opt_present("verbose") {
			settings.verbose = true;
		}

		return Result::Ok(settings);
	}

	pub fn compile_lists(&mut self, case_insensitive: bool) -> Option<String> {
		self.ignore_list = match RegexSetBuilder::new(&self.ignore_list_str).case_insensitive(case_insensitive).build() {
			Result::Ok(rs) => rs,
			Result::Err(err) => {
				return Option::Some(format!("Error while processing ignore-list patterns: {}", err));
			}
		};
		self.override_list = match RegexSetBuilder::new(&self.override_list_str).case_insensitive(case_insensitive).build() {
			Result::Ok(rs) => rs,
			Result::Err(err) => {
				return Option::Some(format!("Error while processing override-list patterns: {}", err));
			}
		};

		return Option::None;
	}
}
