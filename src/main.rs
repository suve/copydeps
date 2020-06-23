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

extern crate same_file;
use same_file::is_same_file;

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

fn walk_deps_recursively(obj: &Object, settings: &Settings) -> HashMap<String, Status> {
	let mut result: HashMap<String, Status> = HashMap::new();

	let mut unresolved: Vec<String> = obj.deps.clone();
	while !unresolved.is_empty() {
		let entry = unresolved.pop().unwrap();
		if result.contains_key(entry.as_str()) { continue; }

		let status = resolve(&entry, &obj.type_, settings);
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

fn dep_print(name: &String, status: &Status, _settings: &Settings) -> bool {
	match status {
		Status::Ignored => println!("\"{}\": (ignored)", name),
		Status::FailedToResolve => println!("\"{}\": (failed to resolve)", name),
		Status::Resolved(r) => println!("\"{}\": {}", name, r.to_string_lossy()),
	}
	return true;
}

fn should_copy(name: &String, source: &PathBuf, destination: &PathBuf, settings: &Settings) -> Result<bool, String> {
	if !destination.exists() {
		return Result::Ok(true);
	}

	if settings.no_clobber {
		if settings.verbose {
			println!("\"{}\": already exists in the target directory and --no-clobber was specified", name);
		}
		return Result::Ok(false);
	}

	match is_same_file(source, &destination) {
		Ok(true) => {
			if settings.verbose {
				println!("\"{}\": preferred version already present in target directory", name);
			}
			return Result::Ok(false);
		},
		Err(err) => {
			return Result::Err(format!(
				"Failed to determine if \"{}\" and \"{}\" refer to the same file: {}",
				source.to_string_lossy(), destination.to_string_lossy(), err
			));
		}
		Ok(false) => { return Result::Ok(true); }
	};
}

fn dep_copy(name: &String, status: &Status, settings: &Settings) -> bool {
	match status {
		Status::Ignored => {
			if settings.verbose {
				println!("\"{}\": ignored, skipping", name)
			}
			return true;
		},
		Status::FailedToResolve => {
			eprintln!("{}: failed to resolve \"{}\"", PROGRAM_NAME, name);
			return false;
		},
		Status::Resolved(resolved) => {
			let mut destination = settings.target_dir.clone();
			destination.push(name);

			match should_copy(name, resolved, &destination, settings) {
				Err(err) => {
					eprintln!("{}: {}", PROGRAM_NAME, err);
					return false;
				},
				Ok(false) => {
					return true;
				},
				Ok(true) => {
					match fs::copy(resolved, &destination) {
						Ok(_) => {
							if settings.verbose {
								println!("\"{}\": {} -> {}", name, resolved.to_string_lossy(), destination.to_string_lossy())
							}
							return true;
						},
						Err(err) => {
							eprintln!("{}: failed to copy \"{}\": {}", PROGRAM_NAME, name, err);
							return false;
						},
					}
				}
			}
		}
	}
}

type DepCallback = fn(name: &String, status: &Status, settings: &Settings) -> bool;

fn process_deps(deps: &HashMap<String, Status>, callback: DepCallback, settings: &Settings) -> bool {
	let mut all_ok = true;

	let mut sorted_keys = deps.keys().collect::<Vec<&String>>();
	sorted_keys.sort();

	for key in sorted_keys {
		let val = deps.get(key.as_str()).unwrap();
		if !callback(&key, val, settings) {
			all_ok = false;
		}
	}

	return all_ok;
}

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

	let deps = walk_deps_recursively(&executable, &settings);
	let status = process_deps(&deps, if settings.dry_run { dep_print } else { dep_copy }, &settings);
	exit(if status { 0 } else { 4 });
}
