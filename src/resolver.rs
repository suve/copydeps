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
use std::path::PathBuf;

extern crate goblin;
use goblin::Object;

use crate::parser::ObjectType;

pub fn resolve(name: &String, type_: &ObjectType) -> Option<String> {
	let mut search_paths = match type_ {
		ObjectType::Elf32 => vec!["/lib/", "/usr/lib/", "/usr/local/lib/"],
		ObjectType::Elf64 => vec!["/lib64/", "/usr/lib64/", "/usr/local/lib64/"],
		ObjectType::Exe32 => vec!["/usr/i686-w64-mingw32/sys-root/mingw/bin/"],
		ObjectType::Exe64 => vec!["/usr/x86_64-w64-mingw32/sys-root/mingw/bin/"],
	};

	for dir in search_paths {
		let path = PathBuf::from(dir.to_owned() + &name);
		if path.exists() {
			return Option::Some(String::from(path.to_str().unwrap()));
		}
	}

	return Option::None;
}
