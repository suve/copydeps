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
use std::path::PathBuf;

use crate::parser::ObjectType;
use crate::settings::Settings;

pub enum Status {
	Ignored,
	FailedToResolve,
	Resolved(String)
}

fn find_in_directory(name: &String, type_: &ObjectType, dir: &str) -> Option<String> {
	match type_ {
		// With ELF, look for an exact match.
		ObjectType::Elf32 | ObjectType::Elf64 => {
			let filepath = PathBuf::from(dir.to_owned() + name);
			if filepath.exists() {
				return Option::Some(name.parse().unwrap())
			}
		},
		// With PE, iterate over the directory entries and look for a case-insensitive match.
		ObjectType::Exe32 | ObjectType::Exe64 => {
			if let Ok(entries) = fs::read_dir(dir) {
				for entry in entries {
					if let Ok(entry) = entry {
						match entry.file_name().to_str() {
							Some(entry_name) => {
								if name.eq_ignore_ascii_case(entry_name) {
									return Option::Some(String::from(entry_name))
								}
							},
							None => { /* ignore */ }
						}
					}
				}
			}
		}
	}

	return Option::None;
}

fn exists_in_list(name: &String, type_: &ObjectType, list: &Vec<&str>) -> bool {
	match type_ {
		// Use exact match for .so
		ObjectType::Elf32 | ObjectType::Elf64 => {
			for entry in list {
				if name == entry {
					return true;
				}
			}
		},
		// Perform case-insensitive matching for .dll
		ObjectType::Exe32 | ObjectType::Exe64 => {
			for entry in list {
				if name.eq_ignore_ascii_case(entry) {
					return true;
				}
			}
		}
	}

	return false;
}

fn exists_in_ignore_list(name: &String, type_: &ObjectType, settings: &Settings) -> bool {
	if exists_in_list(name, type_, &settings.ignore_list.iter().map(|item| item.as_str()).collect()) {
		return true;
	}

	let ignore_list = match type_ {
		ObjectType::Elf32 => vec![
			"ld-linux.so"
		],
		ObjectType::Elf64 => vec![
			"ld-linux-x86-64.so"
		],
		ObjectType::Exe32 | ObjectType::Exe64 => vec![
			"ADVAPI32.dll",
			"CRYPT32.dll",
			"GDI32.dll",
			"IMM32.dll",
			"KERNEL32.dll",
			"msvcrt.dll",
			"ole32.dll", "OLEAUT32.dll",
			"SETUPAPI.dll", "SHELL32.dll",
			"USER32.dll",
			"VERSION.dll",
			"WINMM.dll", "WS2_32.dll"
		],
	};
	return exists_in_list(name, type_, &ignore_list);
}

pub fn resolve(name: &String, type_: &ObjectType, settings: &Settings) -> Status {
	if !exists_in_list(name, type_, &settings.override_list.iter().map(|item| item.as_str()).collect()) {
		if exists_in_ignore_list(name, type_, settings) {
			return Status::Ignored;
		}
	}

	let search_paths = match type_ {
		ObjectType::Elf32 => vec!["/lib/", "/usr/lib/", "/usr/local/lib/"],
		ObjectType::Elf64 => vec!["/lib64/", "/usr/lib64/", "/usr/local/lib64/"],
		ObjectType::Exe32 => vec!["/usr/i686-w64-mingw32/sys-root/mingw/bin/"],
		ObjectType::Exe64 => vec!["/usr/x86_64-w64-mingw32/sys-root/mingw/bin/"],
	};

	for dir in search_paths {
		match find_in_directory(&name, &type_, &dir) {
			Some(resolved) => return Status::Resolved(format!("{}{}", dir, resolved)),
			None => { /* do nothing */ }
		}
	}

	return Status::FailedToResolve;
}
