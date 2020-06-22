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
use std::fs;
use std::path::PathBuf;
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

fn walk_deps_recursively(obj: &Object) -> HashMap<String, Status> {
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

fn print_deps(deps: &HashMap<String, Status>) {
	for (key, val) in deps {
		match val {
			Status::Ignored => println!("\"{}\": (ignored)", key),
			Status::FailedToResolve => println!("\"{}\": (failed to resolve)", key),
			Status::Resolved(r) => println!("\"{}\": {}", key, r),
		}
	}
}

fn copy_deps(deps: &HashMap<String, Status>, settings: &Settings) {
	for (key, val) in deps {
		match val {
			Status::Ignored => println!("\"{}\": ignored, skipping", key),
			Status::FailedToResolve => println!("\"{}\": failed to resolve", key),
			Status::Resolved(resolved) => {
				let destination = PathBuf::from(format!("{}/{}", settings.target_dir, key));

				if (settings.no_clobber) && (destination.exists()) {
					println!("\"{}\": already exists in the target directory and --no-clobber was specified", key);
					continue;
				}

				match fs::copy(resolved, &destination) {
					Ok(_) => println!("\"{}\": {} -> {}", key, resolved, destination.to_str().unwrap()),
					Err(err) => println!("\"{}\" could not be copied: {}", key, err),
				}
			}
		}
	}
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

	let deps = walk_deps_recursively(&executable);
	if settings.dry_run {
		print_deps(&deps);
	} else {
		copy_deps(&deps, &settings);
	}
}
