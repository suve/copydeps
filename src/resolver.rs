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

extern crate goblin;
use goblin::Object;

use crate::parser::ObjectType;

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

pub fn resolve(name: &String, type_: &ObjectType) -> Option<String> {
	let mut search_paths = match type_ {
		ObjectType::Elf32 => vec!["/lib/", "/usr/lib/", "/usr/local/lib/"],
		ObjectType::Elf64 => vec!["/lib64/", "/usr/lib64/", "/usr/local/lib64/"],
		ObjectType::Exe32 => vec!["/usr/i686-w64-mingw32/sys-root/mingw/bin/"],
		ObjectType::Exe64 => vec!["/usr/x86_64-w64-mingw32/sys-root/mingw/bin/"],
	};

	for dir in search_paths {
		match find_in_directory(&name, &type_, &dir) {
			Some(s) => return Option::Some(s),
			None => { /* do nothing */ }
		}
	}

	return Option::None;
}
