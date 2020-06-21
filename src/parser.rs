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
use std::vec::Vec;

extern crate goblin;
use goblin::Object;
use goblin::elf::Elf;
use goblin::pe::PE;

fn get_deps_elf(elf: Elf) -> Result<Vec<String>, String>  {
	let mut list: Vec<String> = vec![];
	for entry in elf.libraries {
		list.push(String::from(entry));
	}

	return Result::Ok(list);
}

fn get_deps_pe(exe: PE) -> Result<Vec<String>, String>  {
	let mut list: Vec<String> = vec![];
	for entry in exe.libraries {
		list.push(String::from(entry));
	}

	return Result::Ok(list);
}

pub fn get_deps(filename: &String) -> Result<Vec<String>, String> {
	let bytes = match fs::read(filename) {
		Ok(bytes) => bytes,
		Err(msg) => { return Result::Err(format!("Failed to open file \"{}\": {}", filename, msg)); }
	};

	let object = match Object::parse(&bytes) {
		Ok(obj) => obj,
		Err(msg) => { return Result::Err(format!("Failed to parse file \"{}\": {}", filename, msg)); }
	};

	match object {
		Object::Elf(elf) => return get_deps_elf(elf),
		Object::PE(pe) => return get_deps_pe(pe),
		Object::Mach(_) => return Result::Err(format!("File \"{}\" is an unsupported object type \"Mach\"", filename)),
		Object::Archive(_) => return Result::Err(format!("File \"{}\" is an unsupported object type \"Archive\"", filename)),
		Object::Unknown(magic) => return Result::Err(format!("File \"{}\" is an unsupported object type (magic: {})", filename, magic)),
	}
}
