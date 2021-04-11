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
use std::fs;
use std::path::Path;
use std::vec::Vec;

extern crate goblin;
use goblin::elf::Elf;
use goblin::pe::PE;
use goblin::Object as Goblin;

pub enum ObjectType {
	Elf32,
	Elf64,
	Exe32,
	Exe64,
}

impl ObjectType {
	pub fn is_exe(&self) -> bool {
		match self {
			ObjectType::Exe32 | ObjectType::Exe64 => return true,
			_ => return false,
		}
	}
}

pub struct Object {
	pub type_: ObjectType,
	pub deps: Vec<String>,
}

fn get_deps_elf(elf: Elf) -> Object {
	return Object {
		type_: if elf.is_64 {
			ObjectType::Elf64
		} else {
			ObjectType::Elf32
		},
		deps: elf
			.libraries
			.iter()
			.map(|item| String::from(*item))
			.collect(),
	};
}

fn get_deps_pe(exe: PE) -> Object {
	return Object {
		type_: if exe.is_64 {
			ObjectType::Exe64
		} else {
			ObjectType::Exe32
		},
		deps: exe
			.libraries
			.iter()
			.map(|item| String::from(*item))
			.collect(),
	};
}

pub fn get_deps(filename: &Path) -> Result<Object, String> {
	let bytes = match fs::read(filename) {
		Ok(bytes) => bytes,
		Err(msg) => {
			return Err(format!(
				"Failed to open file \"{}\": {}",
				filename.to_string_lossy(),
				msg
			));
		}
	};

	let object = match Goblin::parse(&bytes) {
		Ok(obj) => obj,
		Err(msg) => {
			return Err(format!(
				"Failed to parse file \"{}\": {}",
				filename.to_string_lossy(),
				msg
			));
		}
	};

	match object {
		Goblin::Elf(elf) => return Ok(get_deps_elf(elf)),
		Goblin::PE(pe) => return Ok(get_deps_pe(pe)),
		Goblin::Mach(_) => {
			return Err(format!(
				"File \"{}\" is an unsupported object type \"Mach\"",
				filename.to_string_lossy()
			))
		}
		Goblin::Archive(_) => {
			return Err(format!(
				"File \"{}\" is an unsupported object type \"Archive\"",
				filename.to_string_lossy()
			))
		}
		Goblin::Unknown(magic) => {
			return Err(format!(
				"File \"{}\" is an unsupported object type (magic: {})",
				filename.to_string_lossy(),
				magic
			))
		}
	}
}
