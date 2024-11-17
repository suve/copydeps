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
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

extern crate regex;
use regex::RegexSet;
use regex::RegexSetBuilder;

use crate::{
	parser::{get_deps, GetDepsError, Object, ObjectType},
	settings::Settings,
};

pub enum Status {
	Ignored,
	FailedToResolve,
	Resolved(PathBuf),
}

fn find_in_directory(name: &str, type_: ObjectType, dir: &Path) -> Option<String> {
	match type_ {
		// With ELF, look for an exact match.
		ObjectType::Elf32 | ObjectType::Elf64 => {
			let mut filepath = PathBuf::from(dir);
			filepath.push(name);

			if filepath.exists() {
				return Some(name.to_owned());
			}
		}
		// With PE, iterate over the directory entries and look for a case-insensitive match.
		ObjectType::Exe32 | ObjectType::Exe64 => {
			if let Ok(entries) = fs::read_dir(dir) {
				for entry in entries {
					if let Ok(entry) = entry {
						if let Some(entry_name) = entry.file_name().to_str() {
							if name.eq_ignore_ascii_case(entry_name) {
								return Some(String::from(entry_name));
							}
						}
					}
				}
			}
		}
	}

	None
}

fn build_elf_ignorelist() -> RegexSet {
	// This regex tries to catch all the names for ld-linux found in glibc.
	RegexSetBuilder::new(vec![r"^ld-linux(?:|-[a-zA-Z0-9_\-]+)\.so\.[0-9.]*$"])
		.build()
		.unwrap()
}

fn build_exe_ignorelist() -> RegexSet {
	RegexSetBuilder::new(vec![
		r"^ADVAPI32\.dll$",
		r"^COMCTL32\.dll$",
		r"^COMDLG32\.dll$",
		r"^CRYPT32\.dll$",
		r"^GDI32\.dll$",
		r"^IMM32\.dll$",
		r"^KERNEL32\.dll$",
		r"^msvcrt\.dll$",
		r"^ncrypt\.dll$",
		r"^NETAPI32\.dll$",
		r"^NTDLL\.dll$",
		r"^ole32\.dll$",
		r"^OLEAUT32\.dll$",
		r"^Secur32\.dll$",
		r"^SETUPAPI\.dll$",
		r"^SHSCRAP\.dll$",
		r"^SHELL32\.dll$",
		r"^USER32\.dll$",
		r"^UserEnv\.dll$",
		r"^VERSION\.dll$",
		r"^WINMM\.dll$",
		r"^WLDAP32\.dll$",
		r"^WS2_32\.dll$",
	])
	.case_insensitive(true)
	.build()
	.unwrap()
}

static IGNORELIST_ELF: OnceLock<RegexSet> = OnceLock::new();
static IGNORELIST_EXE: OnceLock<RegexSet> = OnceLock::new();

fn exists_in_ignore_list(name: &str, type_: ObjectType, settings: &Settings) -> bool {
	if settings.ignore_list.is_match(name) {
		return true;
	}

	let builtin_ignore_list: &RegexSet = match type_ {
		ObjectType::Elf32 | ObjectType::Elf64 => IGNORELIST_ELF.get_or_init(build_elf_ignorelist),
		ObjectType::Exe32 | ObjectType::Exe64 => IGNORELIST_EXE.get_or_init(build_exe_ignorelist),
	};
	builtin_ignore_list.is_match(name)
}

pub fn resolve(name: &str, type_: ObjectType, settings: &Settings) -> Status {
	if !settings.override_list.is_match(name) {
		if exists_in_ignore_list(name, type_, settings) {
			return Status::Ignored;
		}
	}

	let dir_lists = [
		settings.search_dirs.as_slice(),
		crate::search_paths::get_search_paths(type_),
	];
	for list in dir_lists {
		for dir in list {
			match find_in_directory(name, type_, dir.as_path()) {
				Some(resolved) => {
					let mut path = dir.clone();
					path.push(resolved);
					return Status::Resolved(path);
				}
				None => { /* do nothing */ }
			}
		}
	}

	Status::FailedToResolve
}

pub fn resolve_recursively(
	obj: &Object,
	settings: &Settings,
) -> Result<HashMap<String, Status>, GetDepsError> {
	let mut result: HashMap<String, Status> = HashMap::new();

	let mut unresolved: Vec<String> = obj.deps.clone();
	while let Some(entry) = unresolved.pop() {
		if result.contains_key(entry.as_str()) {
			continue;
		}

		let status = resolve(&entry, obj.type_, settings);
		if let Status::Resolved(path) = &status {
			match get_deps(path) {
				Ok(mut sub_obj) => {
					unresolved.append(&mut sub_obj.deps);
				}
				Err(e) => {
					return Err(e);
				}
			}
		}
		result.insert(entry, status);
	}

	Ok(result)
}
