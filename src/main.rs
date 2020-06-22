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
use std::collections::HashMap;
use std::process::exit;

mod parser;
use parser::Object;
use parser::get_deps;

mod resolver;
use resolver::Status;
use resolver::resolve;

mod settings;
use settings::Settings;

mod version;
use version::*;

fn generate_dependency_list(obj: &Object) -> HashMap<String, Status> {
	let mut result: HashMap<String, Status> = HashMap::new();

	let mut unresolved: Vec<String> = obj.deps.clone();
	while !unresolved.is_empty() {
		let entry = unresolved.pop().unwrap();
		if result.contains_key(entry.as_str()) { continue; }

		let status = resolve(&entry, &obj.type_);
		if let Status::Resolved(path) = &status {
			match get_deps(&path) {
				Ok(mut sub_obj) => { unresolved.append(&mut sub_obj.deps); },
				Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(3); }
			}
		}
		result.insert(entry, status);
	}

	return result;
}

fn main() {
	let settings = match Settings::new_from_argv() {
		Ok(s) => s,
		Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(1); }
	};

	let executable = match get_deps(&settings.executable) {
		Ok(obj) => obj,
		Err(msg) => { eprintln!("{}: {}", PROGRAM_NAME, msg); exit(2); }
	};

	let deps = generate_dependency_list(&executable);
	for (key, val) in deps {
		match val {
			Status::Ignored => println!("\"{}\": (ignored)", key),
			Status::FailedToResolve => println!("\"{}\": (failed to resolve)", key),
			Status::Resolved(r) => println!("\"{}\": {}", key, r),
		}
	}
}
