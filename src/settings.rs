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
use std::vec::Vec;

extern crate getopts;
use getopts::Options;
use getopts::ParsingStyle;

pub struct Settings {
	black_list: Vec<String>,
	dry_run: bool,
	executable: String,
	no_clobber: bool,
	search_dirs: Vec<String>,
	target_dir: String,
	verbose: bool,
	white_list: Vec<String>,
}

impl Settings {
	pub fn new() -> Settings {
		return Settings {
			black_list: vec![],
			dry_run: false,
			executable: String::from(""),
			no_clobber: false,
			search_dirs: vec![],
			target_dir: String::from(""),
			verbose: false,
			white_list: vec![],
		};
	}

	pub fn new_from_argv() -> Result<Settings, String> {
		let args: Vec<String> = env::args().collect();
		let mut settings = Settings::new();

		let mut opts = Options::new();
		opts.parsing_style(ParsingStyle::FloatingFrees);
		opts.long_only(true);

		opts.optmulti("", "blacklist", "", "");
		opts.optmulti("", "search-dir", "", "");
		opts.optmulti("", "whitelist", "", "");

		opts.optflag("", "dry-run", "");
		opts.optflag("", "exedir", "");
		opts.optflag("", "no-clobber", "");
		opts.optflag("", "verbose", "");

		let matches = match opts.parse(args) {
			Ok(m) => { m },
			Err(f) => { return Result::Err(f.to_string()) }
		};

		settings.black_list = matches.opt_strs("blacklist");
		settings.search_dirs = matches.opt_strs("search-dir");
		settings.white_list = matches.opt_strs("whitelist");

		if matches.opt_present("dry-run") {
			settings.dry_run = true;
		}
		if matches.opt_present("exedir") {
			// add dir of executable to search_dirs
		}
		if matches.opt_present("no-clobber") {
			settings.no_clobber = true;
		}
		if matches.opt_present("no-clobber") {
			settings.verbose = true;
		}

		return Result::Ok(settings);
	}
}
