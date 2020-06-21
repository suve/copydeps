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

mod parser;
use parser::get_deps;

mod settings;
use settings::Settings;

mod version;
use version::*;

fn main() {
	let settings = match Settings::new_from_argv() {
		Ok(s) => s,
		Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(1); }
	};

	let deps = match get_deps(&settings.executable) {
		Ok(list) => list,
		Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(2); }
	};
}
