/**
 * This file is part of the copydeps program.
 * Copyright (C) 2020-2021 Artur "suve" Iwicki
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

extern crate same_file;
use same_file::is_same_file;

use crate::resolver::Status;
use crate::settings::Settings;
use crate::version::*;

enum ProcessingStatus {
	Ignored,
	ResolveError,
	Skipped,
	Failed,
	Success,
}

fn should_copy(
	name: &String,
	source: &PathBuf,
	destination: &PathBuf,
	settings: &Settings,
) -> Result<bool, String> {
	if !destination.exists() {
		return Ok(true);
	}

	if settings.no_clobber {
		if settings.verbose {
			println!(
				"\"{}\": already exists in the target directory and --no-clobber was specified",
				name
			);
		}
		return Ok(false);
	}

	match is_same_file(source, &destination) {
		Ok(true) => {
			if settings.verbose {
				println!(
					"\"{}\": preferred version already present in target directory",
					name
				);
			}
			return Ok(false);
		}
		Err(err) => {
			return Err(format!(
				"Failed to determine if \"{}\" and \"{}\" refer to the same file: {}",
				source.to_string_lossy(),
				destination.to_string_lossy(),
				err
			));
		}
		Ok(false) => {
			return Ok(true);
		}
	};
}

fn dep_copy(name: &String, status: &Status, settings: &Settings) -> ProcessingStatus {
	match status {
		Status::Ignored => {
			if settings.verbose {
				println!("\"{}\": ignored, skipping", name)
			}
			return ProcessingStatus::Ignored;
		}
		Status::FailedToResolve => {
			eprintln!("{}: failed to resolve \"{}\"", PROGRAM_NAME, name);
			return ProcessingStatus::ResolveError;
		}
		Status::Resolved(resolved) => {
			let mut destination = settings.target_dir.clone();
			destination.push(name);

			match should_copy(name, resolved, &destination, settings) {
				Err(err) => {
					eprintln!("{}: {}", PROGRAM_NAME, err);
					return ProcessingStatus::Failed;
				}
				Ok(false) => {
					return ProcessingStatus::Skipped;
				}
				Ok(true) => match fs::copy(resolved, &destination) {
					Ok(_) => {
						if settings.verbose {
							println!(
								"\"{}\": {} -> {}",
								name,
								resolved.to_string_lossy(),
								destination.to_string_lossy()
							)
						}
						return ProcessingStatus::Success;
					}
					Err(err) => {
						eprintln!("{}: failed to copy \"{}\": {}", PROGRAM_NAME, name, err);
						return ProcessingStatus::Failed;
					}
				},
			}
		}
	}
}

fn dep_print(name: &String, status: &Status, _settings: &Settings) -> ProcessingStatus {
	match status {
		Status::Ignored => {
			println!("\"{}\": (ignored)", name);
			return ProcessingStatus::Ignored;
		}
		Status::FailedToResolve => {
			println!("\"{}\": (failed to resolve)", name);
			return ProcessingStatus::ResolveError;
		}
		Status::Resolved(r) => {
			println!("\"{}\": {}", name, r.to_string_lossy());
			return ProcessingStatus::Success;
		}
	}
}

type DepCallback = fn(name: &String, status: &Status, settings: &Settings) -> ProcessingStatus;

fn process_deps(
	deps: &HashMap<String, Status>,
	callback: DepCallback,
	settings: &Settings,
) -> ProcessingResult {
	let mut result = ProcessingResult {
		failed_to_resolve: 0,
		failed_to_copy: 0,
		successful: 0,
	};

	let mut sorted_keys = deps.keys().collect::<Vec<&String>>();
	sorted_keys.sort();

	for key in sorted_keys {
		let val = deps.get(key.as_str()).unwrap();
		match callback(&key, val, settings) {
			ProcessingStatus::ResolveError => result.failed_to_resolve += 1,
			ProcessingStatus::Failed => result.failed_to_copy += 1,
			_ => result.successful += 1,
		}
	}

	return result;
}

pub struct ProcessingResult {
	pub failed_to_resolve: i32,
	pub failed_to_copy: i32,
	pub successful: i32,
}

pub fn copy_deps(deps: &HashMap<String, Status>, settings: &Settings) -> ProcessingResult {
	return process_deps(deps, dep_copy, settings);
}

pub fn print_deps(deps: &HashMap<String, Status>, settings: &Settings) -> ProcessingResult {
	return process_deps(deps, dep_print, settings);
}
