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
use goblin::Object as Goblin;
use goblin::elf::Elf;
use goblin::pe::PE;

pub enum ObjectType {
	Elf32,
	Elf64,
	Exe32,
	Exe64
}

pub struct Object {
	pub type_: ObjectType,
	pub deps: Vec<String>,
}

fn get_deps_elf(elf: Elf) -> Result<Object, String>  {
	let mut list: Vec<String> = vec![];
	for entry in elf.libraries {
		list.push(String::from(entry));
	}

	let type_ = if elf.is_64 { ObjectType::Elf64 } else { ObjectType::Elf32 };

	return Result::Ok(Object{
		type_: type_,
		deps: list,
	});
}

fn get_deps_pe(exe: PE) -> Result<Object, String>  {
	let mut list: Vec<String> = vec![];
	for entry in exe.libraries {
		list.push(String::from(entry));
	}

	let type_ = if exe.is_64 { ObjectType::Exe64 } else { ObjectType::Exe32 };

	return Result::Ok(Object{
		type_: type_,
		deps: list,
	});
}

pub fn get_deps(filename: &String) -> Result<Object, String> {
	let bytes = match fs::read(filename) {
		Ok(bytes) => bytes,
		Err(msg) => { return Result::Err(format!("Failed to open file \"{}\": {}", filename, msg)); }
	};

	let object = match Goblin::parse(&bytes) {
		Ok(obj) => obj,
		Err(msg) => { return Result::Err(format!("Failed to parse file \"{}\": {}", filename, msg)); }
	};

	match object {
		Goblin::Elf(elf) => return get_deps_elf(elf),
		Goblin::PE(pe) => return get_deps_pe(pe),
		Goblin::Mach(_) => return Result::Err(format!("File \"{}\" is an unsupported object type \"Mach\"", filename)),
		Goblin::Archive(_) => return Result::Err(format!("File \"{}\" is an unsupported object type \"Archive\"", filename)),
		Goblin::Unknown(magic) => return Result::Err(format!("File \"{}\" is an unsupported object type (magic: {})", filename, magic)),
	}
}
