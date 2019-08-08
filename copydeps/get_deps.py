# -*- coding: utf-8 -*-
#
# This file is part of the copydeps program.
# Copyright (C) 2019 Artur "suve" Iwicki
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License,
# either version 3 of the License, or (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with
# this program (LICENCE.txt). If not, see <https://www.gnu.org/licenses/>.
#

from enum import Enum
import os
import re
import sys

from copydeps.run_program import run
from copydeps.settings import settings
from copydeps.version import PROGRAM_NAME


FileFormat = Enum("FileFormat", "elf32 elf64 win32 win64")


class Dependency:
	name = None
	format = None
	path = None
	isBlacklisted = False

	def __check_user_blacklist(self):
		for white_re in settings.whitelist:
			if white_re.match(self.name):
				return False

		for black_re in settings.blacklist:
			if black_re.match(self.name):
				return True

		return None

	def __check_builtin_blacklist__(self):
		if self.format == FileFormat.elf32:
			blacklist = [r"^ld-linux\.so*"]
		elif self.format == FileFormat.elf64:
			blacklist = [r"^ld-linux-x86-64\.so*"]
		elif self.format in [FileFormat.win32, FileFormat.win64]:
			blacklist = [
				r"^ADVAPI32\.dll$",
				r"^CRYPT32\.dll$",
				r"^GDI32\.dll$",
				r"^IMM32\.dll$",
				r"^KERNEL32\.dll$",
				r"^msvcrt\.dll$",
				r"^ole32\.dll$", r"^OLEAUT32\.dll$",
				r"^SETUPAPI\.dll$", r"^SHELL32\.dll$",
				r"^USER32\.dll$",
				r"^VERSION\.dll$",
				r"^WINMM\.dll$", r"^WS2_32\.dll$"]
		else:
			blacklist = []

		for black_pattern in blacklist:
			if re.match(black_pattern, self.name):
				return True

		return False

	def __check_blacklist__(self):
		user = self.__check_user_blacklist()
		if user is not None:
			return user
		return self.__check_builtin_blacklist__()

	def resolve(self):
		if settings.exedir:
			exe_dirs = [os.path.dirname(settings.executable) + "/"]
		else:
			exe_dirs = []

		user_dirs = settings.search_dirs

		if self.format == FileFormat.elf32:
			sys_dirs = ["/lib/", "/usr/lib/", "/usr/local/lib/"]
		elif self.format == FileFormat.elf64:
			sys_dirs = ["/lib64/", "/usr/lib64/", "/usr/local/lib64/"]
		elif self.format == FileFormat.win32:
			sys_dirs = ["/usr/i686-w64-mingw32/sys-root/mingw/bin/"]
		elif self.format == FileFormat.win64:
			sys_dirs = ["/usr/x86_64-w64-mingw32/sys-root/mingw/bin/"]
		else:
			sys_dirs = []

		search_dirs = exe_dirs + user_dirs + sys_dirs

		for dirname in search_dirs:
			path = dirname + self.name
			if os.path.isfile(path):
				self.path = path
				return path
		return None

	def __init__(self, name, format):
		self.format = format
		if format not in [FileFormat.elf32, FileFormat.elf64, FileFormat.win32, FileFormat.win64]:
			print(PROGRAM_NAME + ": incorrect file format value (" + format + "), something is very wrong", file=sys.stderr)
			sys.exit(1)

		self.name = name
		self.isBlacklisted = self.__check_blacklist__()


def get_deps__parse_header(executable, header):
	file_format = None
	for line in header:
		if " file format " not in line:
			continue

		parts = line.split(" file format ")
		format_str = parts[1].strip()

		if format_str[:6] == "elf32-":
			return FileFormat.elf32
		if format_str[:6] == "elf64-":
			return FileFormat.elf64

		if format_str == "pei-i386":
			return FileFormat.win32
		if format_str == "pei-x86-64":
			return FileFormat.win64

		print(PROGRAM_NAME + ": unrecognized file format \"" + format_str + "\" (file: \"" + executable + "\")", file=sys.stderr)
		sys.exit(1)

	if file_format is None:
		print(PROGRAM_NAME + ": could not determine file format for \"" + executable + "\"", file=sys.stderr)
		sys.exit(1)


def get_deps__parse_line(line, file_format):
	if file_format in [FileFormat.elf32, FileFormat.elf64]:
		if "  NEEDED  " not in line:
			return None

		parts = line.split(" ")
		so_name = parts[len(parts)-1].strip()

		return so_name
	elif file_format in [FileFormat.win32, FileFormat.win64]:
		if "\tDLL Name: " not in line:
			return None

		parts = line.split("\tDLL Name: ")
		so_name = parts[1].strip()

		return so_name
	else:
		print(PROGRAM_NAME + ": unknown file_format value (" + file_format + "), something is very wrong", file=sys.stderr)
		sys.exit(1)


def get_deps_recursive(executable, deplist):
	code, output, err = run("objdump", ["-x", executable])
	if code != 0:
		print(PROGRAM_NAME + ": \"objdump\" returned an error\n" + err[0], file=sys.stderr)
		sys.exit(1)

	header = output[:5]
	output = output[5:]

	file_format = get_deps__parse_header(executable, header)

	for line in output:
		so_name = get_deps__parse_line(line, file_format)
		if so_name is None:
			continue
		if so_name in deplist:
			continue

		dep = Dependency(so_name, file_format)
		deplist[so_name] = dep

		if not dep.isBlacklisted:
			dep.resolve()
			if dep.path is not None:
				get_deps_recursive(dep.path, deplist)


def get_deps(executable):
	deps = {}
	get_deps_recursive(executable, deps)
	return deps.values()