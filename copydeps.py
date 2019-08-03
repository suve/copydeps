#!/usr/bin/python3
# -*- coding: utf-8 -*-
#
# copydeps - copy a program's dependencies (.so / .dll files)
# Copyright (C) 2017, 2019 Artur "suve" Iwicki
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
# this program (LICENCE.txt). If not, see <http://www.gnu.org/licenses/>.
#

import os
import subprocess
import sys


PROGRAM_AUTHOR  = "suve"
PROGRAM_NAME    = "copydeps"
PROGRAM_VERSION = "3.0"

FILE_FORMAT_ELF32 = 10
FILE_FORMAT_ELF64 = 11
FILE_FORMAT_WIN32 = 20
FILE_FORMAT_WIN64 = 21


def run(program, args = []):
	args.insert(0, program)
	proc = subprocess.run(args=args, capture_output=True)

	status = proc.returncode
	stdout = proc.stdout.decode("utf-8").split("\n")
	stderr = proc.stderr.decode("utf-8").split("\n")

	return status, stdout, stderr


def find_so(soname, file_format):
	if file_format == FILE_FORMAT_ELF32:
		prefixes = ["/lib/", "/usr/lib/", "/usr/local/lib/"]
	elif file_format == FILE_FORMAT_ELF64:
		prefixes = ["/lib64/", "/usr/lib64/", "/usr/local/lib64/"]
	elif file_format == FILE_FORMAT_WIN32:
		prefixes = ["/usr/i686-w64-mingw32/sys-root/mingw/bin/"]
	elif file_format == FILE_FORMAT_WIN64:
		prefixes = ["/usr/x86_64-w64-mingw32/sys-root/mingw/bin/"]
	else:
		print(PROGRAM_NAME + ": unknown file_format value (" + file_format + "), something is very wrong", file=sys.stderr)
		sys.exit(1)

	for prefix in prefixes:
		path = prefix + soname
		if os.path.isfile(path):
			return path

	return None


def get_deps__parse_header(executable, header):
	file_format = None
	for line in header:
		if " file format " not in line:
			continue

		parts = line.split(" file format ")
		format_str = parts[1].strip()

		if format_str[:6] == "elf32-":
			return FILE_FORMAT_ELF32
		if format_str[:6] == "elf64-":
			return FILE_FORMAT_ELF64

		if format_str == "pei-i386":
			return FILE_FORMAT_WIN32
		if format_str == "pei-x86-64":
			return FILE_FORMAT_WIN64

		print(PROGRAM_NAME + ": unrecognized file format \"" + format_str + "\" (file: \"" + executable + "\")", file=sys.stderr)
		sys.exit(1)

	if file_format is None:
		print(PROGRAM_NAME + ": could not determine file format for \"" + executable + "\"", file=sys.stderr)
		sys.exit(1)


def get_deps__parse_line(line, file_format):
	if file_format in [FILE_FORMAT_ELF32, FILE_FORMAT_ELF64]:
		if "  NEEDED  " not in line:
			return None

		parts = line.split(" ")
		so_name = parts[len(parts)-1].strip()

		return so_name
	elif file_format in [FILE_FORMAT_WIN32, FILE_FORMAT_WIN64]:
		if "\tDLL Name: " not in line:
			return None

		parts = line.split("\tDLL Name: ")
		so_name = parts[1].strip()

		return so_name
	else:
		print(PROGRAM_NAME + ": unknown file_format value (" + file_format + "), something is very wrong", file=sys.stderr)
		sys.exit(1)


def check_blacklist(executable, file_format):
	if file_format == FILE_FORMAT_ELF32:
		blacklist = ["ld-linux."]
	elif file_format == FILE_FORMAT_ELF64:
		blacklist = ["ld-linux-x86-64."]
	elif file_format in [FILE_FORMAT_WIN32, FILE_FORMAT_WIN64]:
		blacklist = [
			"ADVAPI32.dll", "GDI32.dll", "IMM32.dll", "KERNEL32.dll", "msvcrt.dll", "ole32.dll", "OLEAUT32.dll",
			"SETUPAPI.dll", "SHELL32.dll", "USER32.dll", "VERSION.dll", "WINMM.dll", "WS2_32.dll"]
	else:
		print(PROGRAM_NAME + ": unknown file_format value (" + file_format + "), something is very wrong", file=sys.stderr)
		sys.exit(1)

	for blackentry in blacklist:
		if blackentry in executable:
			return True

	return False


def get_deps_recursive(executable, deps):
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
		if so_name in deps:
			continue

		if check_blacklist(so_name, file_format):
			deps[so_name] = None
			print(PROGRAM_NAME + ": \"" + so_name + "\" is blacklisted, skipping")
			continue

		so_path = find_so(so_name, file_format)
		if so_path is None:
			print(PROGRAM_NAME + ": unable to resolve \"" + so_name + "\"", file=sys.stderr)
			sys.exit(1)

		deps[so_name] = so_path
		get_deps_recursive(so_path, deps)


def get_deps(executable):
	deps = {}
	get_deps_recursive(executable, deps)

	return deps


def copy_deps(deps, target_dir):
	for key, value in deps.items():
		so_name = key
		so_path = value

		if so_path is None:
			continue

		code, _, err = run("cp", ["--preserve=timestamps", so_path, target_dir])
		if code == 0:
			print(PROGRAM_NAME + ": \"" + so_name + "\" copied from \"" + so_path + "\"")
		else:
			print(PROGRAM_NAME + ": \"" + so_name + "\" could not be copied (" + err[0] + ")")


def print_help():
	print(
		PROGRAM_NAME + " is a script for bundling the .so / .dll files needed by binary executables.\n"
		"Usage: " + PROGRAM_NAME + " EXECUTABLE [TARGET-DIR]\n"
		"\n"
		"EXECUTABLE can be one of the following supported formats:\n"
		"- 32-bit ELF\n"
		"- 64-bit ELF\n"
		"- i386 Microsoft Windows executable\n"
		"- x86_64 Microsoft Windows executable\n"
		"\n"
		"TARGET-DIR specifies the directory to copy the .so / .dll files to.\n"
		"When omitted, defaults to the current working directory, (!)\n"
		"not to be confused with the directory of the target executable.")


def parse_args():
	argc = len(sys.argv)
	if argc < 2:
		print(PROGRAM_NAME + ": EXECUTABLE is missing\nUsage: " + PROGRAM_NAME + " EXECUTABLE [TARGET-DIR]", file=sys.stderr)
		sys.exit(1)

	if sys.argv[1] == "--help":
		print_help()
		sys.exit(0)

	if sys.argv[1] == "--version":
		print(PROGRAM_NAME + " v." + PROGRAM_VERSION + " by " + PROGRAM_AUTHOR)
		sys.exit(0)

	executable = sys.argv[1]
	if not os.path.isfile(executable):
		print(PROGRAM_NAME + ": File \"" + executable + "\" does not exist", file=sys.stderr)
		sys.exit(1)

	if argc >= 3:
		target_dir = sys.argv[2]
		if not os.path.isdir(target_dir):
			print(PROGRAM_NAME + ": Directory \"" + target_dir + "\" does not exist", file=sys.stderr)
			sys.exit(1)
	else:
		target_dir = os.getcwd()
	target_dir = target_dir + "/"

	return executable, target_dir


def main():
	executable, target_dir = parse_args()
	deps = get_deps(executable)
	copy_deps(deps, target_dir)


if __name__ == "__main__":
	main()
