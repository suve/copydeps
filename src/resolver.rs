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

fn find_in_directory(name: &String, type_: &ObjectType, dir: &Path) -> Option<String> {
	match type_ {
		// With ELF, look for an exact match.
		ObjectType::Elf32 | ObjectType::Elf64 => {
			let mut filepath = PathBuf::from(dir);
			filepath.push(name);

			if filepath.exists() {
				return Some(name.parse().unwrap());
			}
		}
		// With PE, iterate over the directory entries and look for a case-insensitive match.
		ObjectType::Exe32 | ObjectType::Exe64 => {
			if let Ok(entries) = fs::read_dir(dir) {
				for entry in entries {
					if let Ok(entry) = entry {
						match entry.file_name().to_str() {
							Some(entry_name) => {
								if name.eq_ignore_ascii_case(entry_name) {
									return Some(String::from(entry_name));
								}
							}
							None => { /* ignore */ }
						}
					}
				}
			}
		}
	}

	return None;
}

lazy_static! {
	// This regex tries to catch all the names for ld-linux found in glibc.
	static ref IGNORELIST_ELF: RegexSet =
		RegexSetBuilder::new(vec![r"^ld-linux(?:|-[a-zA-Z0-9_\-]+)\.so\.[0-9.]*$",])
			.build()
			.unwrap();

	static ref IGNORELIST_EXE: RegexSet = RegexSetBuilder::new(vec![
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
	.unwrap();
}

fn exists_in_ignore_list(name: &String, type_: &ObjectType, settings: &Settings) -> bool {
	if settings.ignore_list.is_match(name) {
		return true;
	}

	let builtin_ignore_list: &RegexSet = match type_ {
		ObjectType::Elf32 | ObjectType::Elf64 => &IGNORELIST_ELF,
		ObjectType::Exe32 | ObjectType::Exe64 => &IGNORELIST_EXE,
	};
	builtin_ignore_list.is_match(name)
}

// TODO: Think of a way to reduce duplication between /lib and /usr/lib
//       for the Debian triplets.
const SEARCH_PATH_ELF32: &[&str] = &[
	"/lib/",
	"/usr/lib/",
	// Debian triplets (https://wiki.debian.org/Multiarch/Tuples)
	#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
	"/lib/i386-linux-gnu/",
	#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
	"/usr/lib/i386-linux-gnu/",
	#[cfg(target_arch = "arm")]
	"/lib/arm-linux-gnu/",
	#[cfg(target_arch = "arm")]
	"/usr/lib/arm-linux-gnu/",
	#[cfg(target_arch = "arm")]
	"/lib/arm-linux-gnueabi/",
	#[cfg(target_arch = "arm")]
	"/usr/lib/arm-linux-gnueabi/",
	#[cfg(target_arch = "arm")]
	"/lib/arm-linux-gnueabihf/",
	#[cfg(target_arch = "arm")]
	"/usr/lib/arm-linux-gnueabihf/",
	#[cfg(target_arch = "powerpc")]
	"/lib/powerpc-linux-gnu/",
	#[cfg(target_arch = "powerpc")]
	"/usr/lib/powerpc-linux-gnu/",
	// Local
	"/usr/local/lib/",
];

const SEARCH_PATH_ELF64: &[&str] = &[
	"/lib64/",
	"/usr/lib64/",
	// Debian triplets (https://wiki.debian.org/Multiarch/Tuples)
	#[cfg(target_arch = "x86_64")]
	"/lib/x86_64-linux-gnu/",
	#[cfg(target_arch = "x86_64")]
	"/usr/lib/x86_64-linux-gnu/",
	#[cfg(target_arch = "aarch64")]
	"/lib/aarch64-linux-gnu/",
	#[cfg(target_arch = "aarch64")]
	"/usr/lib/aarch64-linux-gnu/",
	#[cfg(all(target_arch = "powerpc64", target_endian = "big"))]
	"/lib/powerpc64-linux-gnu/",
	#[cfg(all(target_arch = "powerpc64", target_endian = "big"))]
	"/usr/lib/powerpc64-linux-gnu/",
	#[cfg(all(target_arch = "powerpc64", target_endian = "little"))]
	"/lib/powerpc64le-linux-gnu/",
	#[cfg(all(target_arch = "powerpc64", target_endian = "little"))]
	"/usr/lib/powerpc64le-linux-gnu/",
	#[cfg(target_arch = "riscv64")]
	"/lib/riscv64-linux-gnu/",
	#[cfg(target_arch = "riscv64")]
	"/usr/lib/riscv64-linux-gnu/",
	#[cfg(target_arch = "s390x")]
	"/lib/s390x-linux-gnu/",
	#[cfg(target_arch = "s390x")]
	"/usr/lib/s390x-linux-gnu/",
	// Local
	"/usr/local/lib64/",
];

const SEARCH_PATH_EXE32: &[&str] = &[
	"/usr/i686-w64-mingw32/lib/",                // Debian
	"/usr/i686-w64-mingw32/sys-root/mingw/bin/", // Fedora
	"/usr/i686-w64-mingw32/sys-root/mingw/lib/", // Maybe used by some other distro?
];

const SEARCH_PATH_EXE64: &[&str] = &[
	"/usr/x86_64-w64-mingw32/lib/",                // Debian
	"/usr/x86_64-w64-mingw32/sys-root/mingw/bin/", // Fedora
	"/usr/x86_64-w64-mingw32/sys-root/mingw/lib/", // Maybe used by some other distro?
];

pub fn resolve(name: &String, type_: &ObjectType, settings: &Settings) -> Status {
	if !settings.override_list.is_match(name) {
		if exists_in_ignore_list(name, type_, settings) {
			return Status::Ignored;
		}
	}

	for dir in &settings.search_dirs {
		match find_in_directory(&name, &type_, dir.as_path()) {
			Some(resolved) => {
				let mut path = dir.clone();
				path.push(resolved);
				return Status::Resolved(path);
			}
			None => { /* do nothing */ }
		}
	}

	let search_paths = match type_ {
		ObjectType::Elf32 => SEARCH_PATH_ELF32,
		ObjectType::Elf64 => SEARCH_PATH_ELF64,
		ObjectType::Exe32 => SEARCH_PATH_EXE32,
		ObjectType::Exe64 => SEARCH_PATH_EXE64,
	};

	// TODO: Detect if /lib{,64} is a symlink to /usr/lib{,64}.
	//       Probably the best solution would be to replace the const lists
	//       with some custom iterator type.
	for dir in search_paths {
		match find_in_directory(&name, &type_, &Path::new(dir)) {
			Some(resolved) => {
				let mut path = PathBuf::from(dir);
				path.push(resolved);
				return Status::Resolved(path);
			}
			None => { /* do nothing */ }
		}
	}

	return Status::FailedToResolve;
}

pub fn resolve_recursively(
	obj: &Object,
	settings: &Settings,
) -> Result<HashMap<String, Status>, GetDepsError> {
	let mut result: HashMap<String, Status> = HashMap::new();

	let mut unresolved: Vec<String> = obj.deps.clone();
	while !unresolved.is_empty() {
		let entry = unresolved.pop().unwrap();
		if result.contains_key(entry.as_str()) {
			continue;
		}

		let status = resolve(&entry, &obj.type_, settings);
		if let Status::Resolved(path) = &status {
			match get_deps(&path) {
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

	return Ok(result);
}
