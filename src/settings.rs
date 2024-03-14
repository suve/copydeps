/**
 * This file is part of the copydeps program.
 * Copyright (C) 2020-2021, 2024 suve (a.k.a. Artur Frenszek-Iwicki)
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
use std::{
	env,
	fmt::{Display, Formatter},
	fs,
	path::{Path, PathBuf},
	process::exit,
	vec::Vec,
};

extern crate getopts;
use getopts::Options;
use getopts::ParsingStyle;

extern crate regex;
use regex::RegexSet;
use regex::RegexSetBuilder;

use crate::exit_status::*;
use crate::version::*;

fn print_help() {
	print!(
		concat!(
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
			"  Add the regular expression PATTERN to the ignore-list\n",
			"  (.so / .dll names that should not be resolved nor copied over).\n",
			"--no-clobber\n",
			"  Do not overwrite .so / .dll files already existing in the target directory.\n",
			"--override PATTERN\n",
			"  Add the regular expression PATTERN to the override-list\n",
			"  (.so / .dll names that should always be resolved and copied over).\n",
			"  Overrides have precedence over ignores.\n",
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
	println!(
		"{} v.{} by {}",
		PROGRAM_NAME, PROGRAM_VERSION, PROGRAM_AUTHOR
	);
}

fn verify_dir(dir: &Path) -> Result<(), SettingsError> {
	match fs::metadata(dir) {
		Ok(meta) => match meta.is_dir() {
			true => Ok(()),
			false => Err(SettingsError::DirectoryNotADirectory(dir.to_path_buf())),
		},
		Err(e) => Err(SettingsError::DirectoryNotFound(dir.to_path_buf(), e)),
	}
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, SettingsError> {
	match path.canonicalize() {
		Ok(value) => Ok(value),
		Err(e) => Err(SettingsError::FailedToCanonicalizePath(
			path.to_path_buf(),
			e,
		)),
	}
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
	// Unfortunately for us, RegexSet does not implement Default
	fn new() -> Settings {
		let empty_vector: Vec<&str> = vec![];
		Settings {
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
		}
	}

	pub fn new_from_argv() -> Result<Settings, SettingsError> {
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

		let matches = opts.parse(args)?;

		if matches.opt_present("help") {
			print_help();
			exit(EXIT_OK);
		}
		if matches.opt_present("version") {
			print_version();
			exit(EXIT_OK);
		}

		match matches.free.len() {
			0 | 1 => return Err(SettingsError::ExecutableNotSpecified),
			2 | 3 => {}
			_ => return Err(SettingsError::TooManyArguments(matches.free.len() - 1)),
		}

		let executable = PathBuf::from(matches.free.get(1).unwrap());
		match fs::metadata(&executable) {
			Ok(meta) => {
				if !meta.is_file() {
					return Err(SettingsError::ExecutableNotAFile(executable));
				}
			}
			Err(e) => return Err(SettingsError::ExecutableNotFound(executable, e)),
		}
		settings.executable = canonicalize_path(&executable)?;

		let executable_dir = settings.executable.parent().unwrap().to_path_buf();

		if matches.free.len() == 3 {
			let target_dir = PathBuf::from(matches.free.get(2).unwrap());
			verify_dir(&target_dir)?;
			settings.target_dir = canonicalize_path(&target_dir)?;
		} else {
			settings.target_dir = executable_dir.clone();
		}

		settings.ignore_list_str = matches.opt_strs("ignore");
		settings
			.ignore_list_str
			.append(matches.opt_strs("blacklist").as_mut());

		settings.override_list_str = matches.opt_strs("override");
		settings
			.override_list_str
			.append(matches.opt_strs("whitelist").as_mut());

		for entry in matches.opt_strs("search-dir") {
			let entry_pb = PathBuf::from(entry);
			verify_dir(&entry_pb)?;
			settings.search_dirs.push(entry_pb);
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

		return Ok(settings);
	}

	pub fn compile_lists(&mut self, case_insensitive: bool) -> Result<(), ListCompilationError> {
		self.ignore_list = match RegexSetBuilder::new(&self.ignore_list_str)
			.case_insensitive(case_insensitive)
			.build()
		{
			Ok(rs) => rs,
			Err(e) => return Err(ListCompilationError::IgnoreList(e)),
		};
		self.override_list = match RegexSetBuilder::new(&self.override_list_str)
			.case_insensitive(case_insensitive)
			.build()
		{
			Ok(rs) => rs,
			Err(e) => return Err(ListCompilationError::OverrideList(e)),
		};

		return Ok(());
	}
}

pub enum SettingsError {
	FailedToParseArguments(getopts::Fail),
	TooManyArguments(usize),
	ExecutableNotSpecified,
	ExecutableNotFound(PathBuf, std::io::Error),
	ExecutableNotAFile(PathBuf),
	DirectoryNotFound(PathBuf, std::io::Error),
	DirectoryNotADirectory(PathBuf),
	FailedToCanonicalizePath(PathBuf, std::io::Error),
}

impl From<getopts::Fail> for SettingsError {
	fn from(value: getopts::Fail) -> Self {
		Self::FailedToParseArguments(value)
	}
}

impl Display for SettingsError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			SettingsError::FailedToParseArguments(e) => {
				write!(f, "Failed to parse arguments: {}", e)
			}
			SettingsError::TooManyArguments(count) => {
				write!(f, "Too many arguments (expected 1 or 2, got {})", count)
			}
			SettingsError::ExecutableNotSpecified => {
				write!(f, "Missing required argument: EXECUTABLE")
			}
			SettingsError::ExecutableNotFound(path, err) => write!(
				f,
				"Failed to access file \"{}\": {}",
				path.to_string_lossy(),
				err
			),
			SettingsError::ExecutableNotAFile(path) => write!(
				f,
				"Path \"{}\" is not a regular file",
				path.to_string_lossy()
			),
			SettingsError::DirectoryNotFound(path, err) => write!(
				f,
				"Failed to access directory \"{}\": {}",
				path.to_string_lossy(),
				err
			),
			SettingsError::DirectoryNotADirectory(path) => {
				write!(f, "Path \"{}\" is not a directory", path.to_string_lossy())
			}
			SettingsError::FailedToCanonicalizePath(path, err) => write!(
				f,
				"Failed to canonicalize path \"{}\": {}",
				path.to_string_lossy(),
				err
			),
		}
	}
}

pub enum ListCompilationError {
	IgnoreList(regex::Error),
	OverrideList(regex::Error),
}

impl Display for ListCompilationError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ListCompilationError::IgnoreList(e) => {
				write!(f, "Error while processing ignore-list patterns: {}", e)
			}
			ListCompilationError::OverrideList(e) => {
				write!(f, "Error while processing override-list patterns: {}", e)
			}
		}
	}
}
