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
use std::{
	fmt::{Display, Formatter},
	fs,
	path::{Path, PathBuf},
	vec::Vec,
};

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

pub enum GetDepsError {
	FailedToOpenFile(PathBuf, std::io::Error),
	FailedToParseFile(PathBuf, goblin::error::Error),
	UnsupportedObjectType(PathBuf, String),
}

impl Display for GetDepsError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			GetDepsError::FailedToOpenFile(path, err) => write!(
				f,
				"Failed to open file \"{}\": {}",
				path.to_string_lossy(),
				err
			),
			GetDepsError::FailedToParseFile(path, err) => write!(
				f,
				"Failed to parse file \"{}\": {}",
				path.to_string_lossy(),
				err
			),
			GetDepsError::UnsupportedObjectType(path, objtype) => write!(
				f,
				"File \"{}\" is an unsupported object type \"{}\"",
				path.to_string_lossy(),
				objtype
			),
		}
	}
}

pub fn get_deps(filename: &Path) -> Result<Object, GetDepsError> {
	let bytes = match fs::read(filename) {
		Ok(bytes) => bytes,
		Err(e) => {
			return Err(GetDepsError::FailedToOpenFile(filename.to_path_buf(), e));
		}
	};

	let object = match Goblin::parse(&bytes) {
		Ok(obj) => obj,
		Err(e) => {
			return Err(GetDepsError::FailedToParseFile(filename.to_path_buf(), e));
		}
	};

	match object {
		Goblin::Elf(elf) => Ok(get_deps_elf(elf)),
		Goblin::PE(pe) => Ok(get_deps_pe(pe)),
		_ => Err(GetDepsError::UnsupportedObjectType(
			filename.to_path_buf(),
			obj_type_name(&object),
		)),
	}
}

fn obj_type_name(obj: &goblin::Object) -> String {
	match obj {
		Goblin::Elf(_) => "Elf".to_string(),
		Goblin::PE(_) => "PE".to_string(),
		Goblin::Mach(_) => "Mach".to_string(),
		Goblin::Archive(_) => "Archive".to_string(),
		Goblin::Unknown(magic) => format!("Unknown (magic: {})", magic),
	}
}
