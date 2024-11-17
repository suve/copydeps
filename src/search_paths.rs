/**
 * This file is part of the copydeps program.
 * Copyright (C) 2024 suve (a.k.a. Artur Frenszek-Iwicki)
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
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::parser::ObjectType;

fn symlink_points_at(src: &str, expected: &str) -> bool {
	match Path::new(src).read_link() {
		Ok(destination) => {
			let absolute = match destination.is_absolute() {
				true => destination,
				false => {
					let mut buf = PathBuf::from(src);
					buf.pop();
					buf.push(destination);
					buf
				}
			};
			absolute == Path::new(expected)
		}
		Err(_) => false,
	}
}

fn push_lib_path(vec: &mut Vec<PathBuf>, path: &str, usr_merged: bool) {
	let path = Path::new(path);
	if !usr_merged && path.is_dir() {
		vec.push(path.to_path_buf());
	}

	let mut buf = PathBuf::from("/usr/");
	match path.strip_prefix("/") {
		Ok(stripped) => buf.push(stripped),
		Err(_) => buf.push(path),
	};
	if buf.is_dir() {
		vec.push(buf);
	}
}

fn generate_elf64_search_paths() -> Vec<PathBuf> {
	let usr_merged = symlink_points_at("/lib64", "/usr/lib64");

	let mut result = Vec::new();
	push_lib_path(&mut result, "/lib64/", usr_merged);

	// Debian triplets (https://wiki.debian.org/Multiarch/Tuples)
	#[cfg(target_arch = "x86_64")]
	push_lib_path(&mut result, "/lib/x86_64-linux-gnu/", usr_merged);
	#[cfg(target_arch = "aarch64")]
	push_lib_path(&mut result, "/lib/aarch64-linux-gnu/", usr_merged);
	#[cfg(all(target_arch = "powerpc64", target_endian = "big"))]
	push_lib_path(&mut result, "/lib/powerpc64-linux-gnu/", usr_merged);
	#[cfg(all(target_arch = "powerpc64", target_endian = "little"))]
	push_lib_path(&mut result, "/lib/powerpc64le-linux-gnu/", usr_merged);
	#[cfg(target_arch = "riscv64")]
	push_lib_path(&mut result, "/lib/riscv64-linux-gnu/", usr_merged);
	#[cfg(target_arch = "s390x")]
	push_lib_path(&mut result, "/lib/s390x-linux-gnu/", usr_merged);

	let local = PathBuf::from("/usr/local/lib64/");
	if local.is_dir() {
		result.push(local);
	}

	result
}

fn generate_elf32_search_paths() -> Vec<PathBuf> {
	let usr_merged = symlink_points_at("/lib", "/usr/lib");

	let mut result = Vec::new();
	push_lib_path(&mut result, "/lib/", usr_merged);

	// Debian triplets (https://wiki.debian.org/Multiarch/Tuples)
	#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
	push_lib_path(&mut result, "/lib/i386-linux-gnu/", usr_merged);
	#[cfg(target_arch = "arm")]
	push_lib_path(&mut result, "/lib/arm-linux-gnu/", usr_merged);
	#[cfg(target_arch = "arm")]
	push_lib_path(&mut result, "/lib/arm-linux-gnueabi/", usr_merged);
	#[cfg(target_arch = "arm")]
	push_lib_path(&mut result, "/lib/arm-linux-gnueabihf/", usr_merged);
	#[cfg(target_arch = "powerpc")]
	push_lib_path(&mut result, "/lib/powerpc-linux-gnu/", usr_merged);

	let local = PathBuf::from("/usr/local/lib/");
	if local.is_dir() {
		result.push(local);
	}

	result
}

fn generate_exe64_search_paths() -> Vec<PathBuf> {
	const PATHS: &[&str] = &[
		"/usr/x86_64-w64-mingw32/lib/",                // Debian
		"/usr/x86_64-w64-mingw32/sys-root/mingw/bin/", // Fedora
		"/usr/x86_64-w64-mingw32/sys-root/mingw/lib/", // Maybe used by some other distro?
	];
	PATHS.iter().map(PathBuf::from).collect()
}

fn generate_exe32_search_paths() -> Vec<PathBuf> {
	const PATHS: &[&str] = &[
		"/usr/i686-w64-mingw32/lib/",                // Debian
		"/usr/i686-w64-mingw32/sys-root/mingw/bin/", // Fedora
		"/usr/i686-w64-mingw32/sys-root/mingw/lib/", // Maybe used by some other distro?
	];
	PATHS.iter().map(PathBuf::from).collect()
}

static ELF64_PATHS: OnceLock<Vec<PathBuf>> = OnceLock::new();
static ELF32_PATHS: OnceLock<Vec<PathBuf>> = OnceLock::new();
static EXE64_PATHS: OnceLock<Vec<PathBuf>> = OnceLock::new();
static EXE32_PATHS: OnceLock<Vec<PathBuf>> = OnceLock::new();

pub fn get_search_paths(objtype: ObjectType) -> &'static [PathBuf] {
	let vec = match objtype {
		ObjectType::Elf64 => ELF64_PATHS.get_or_init(generate_elf64_search_paths),
		ObjectType::Elf32 => ELF32_PATHS.get_or_init(generate_elf32_search_paths),
		ObjectType::Exe64 => EXE64_PATHS.get_or_init(generate_exe64_search_paths),
		ObjectType::Exe32 => EXE32_PATHS.get_or_init(generate_exe32_search_paths),
	};
	vec.as_slice()
}
