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
use std::fs;
use std::process::exit;

extern crate goblin;
use goblin::Object;

mod settings;
use settings::Settings;

mod version;
use version::*;

fn main() {
	let settings = match Settings::new_from_argv() {
		Ok(s) => s,
		Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(1); }
	};

	let bytes = match fs::read(&settings.executable) {
		Ok(bytes) => bytes,
		Err(msg) => { eprintln!("{}: Failed to read file \"{}\": {}", PROGRAM_NAME, &settings.executable, msg); exit(1); }
	};

	let file = match Object::parse(&bytes) {
		Ok(obj) => obj,
		Err(msg) => { eprintln!("{}: Failed to parse file \"{}\": {}", PROGRAM_NAME, &settings.executable, msg); exit(1); }
	};
}
