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
use std::process::exit;

#[macro_use] extern crate lazy_static;

mod parser;
use parser::get_deps;

mod process_deps;
use process_deps::copy_deps;
use process_deps::print_deps;

mod resolver;
use resolver::resolve_recursively;

mod settings;
use settings::Settings;

mod version;
use version::*;


fn main() {
	let mut settings = match Settings::new_from_argv() {
		Ok(s) => s,
		Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(1); }
	};

	let executable = match get_deps(&settings.executable) {
		Ok(obj) => obj,
		Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(2); }
	};

	match settings.compile_lists(executable.type_.is_exe()) {
		Some(err) => { eprintln!("{}: {}", PROGRAM_NAME, err); exit(1); }
		None => { /* do nothing */ }
	}

	let deps = match resolve_recursively(&executable, &settings) {
		Ok(hm) => hm,
		Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(3); }
	};

	let count = match settings.dry_run {
		true => print_deps(&deps, &settings),
		false => copy_deps(&deps, &settings),
	};

	if count.failed_to_resolve > 0 { exit(4); }
	if count.failed_to_copy > 0 { exit(5); }
	exit(0);
}
