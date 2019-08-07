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

import argparse
import os
import re
import sys

from copydeps.version import PROGRAM_AUTHOR, PROGRAM_NAME, PROGRAM_VERSION


class HelpAction(argparse.Action):
	def __call__(self, parser, namespace, values, option_string=None):
		print(
			PROGRAM_NAME + " is a script for bundling the .so / .dll files needed by binary executables.\n"
			"Usage: " + PROGRAM_NAME + " [options...] EXECUTABLE [TARGET-DIR]\n"
			"\n"
			"EXECUTABLE can be one of the following supported formats:\n"
			"- 32-bit ELF\n"
			"- 64-bit ELF\n"
			"- i386 Microsoft Windows executable\n"
			"- x86_64 Microsoft Windows executable\n"
			"\n"
			"TARGET-DIR specifies the directory to copy the .so / .dll files to.\n"
			"When omitted, defaults to the directory of the target executable.\n"
			"\n"
			"Program options:\n"
			"--blacklist PATTERN\n"
			"  Add PATTERN to the built-in blacklist (.so / .dll names that should not\n"
			"  be resolved nor copied over).\n"
			"--dry-run\n"
			"  Print the list of dependencies without actually copying the .so / .dll files.\n"
			"--exedir\n"
			"  Include the directory of the executable in the .so / .dll resolve paths.\n"
			"  Files found in the exedir are preferred over those in system paths.\n"
			"--no-clobber\n"
			"  Do not overwrite .so / .dll files already existing in the target directory.\n"
			"--search-dir DIRECTORY\n"
			"  Add DIRECTORY to the list of paths to search when resolving .so / .dll names.\n"
			"  User-specified directories take precedence over system paths.\n"
			"--verbose\n"
			"  Print the names of the dependencies as they're being copied over.\n"
			"--whitelist PATTERN\n"
			"  Add PATTERN to the whitelist (.so / .dll names that should always be\n"
			"  resolved and copied over). The whitelist has precedence over the blacklist.")
		sys.exit(0)


def compile_regexes(re_list):
	if re_list is None:
		return []

	result = []
	for re_entry in re_list:
		try:
			compiled = re.compile(re_entry)
			result.append(compiled)
		except re.error as err:
			if err.colno is None:
				print(
					PROGRAM_NAME + ": \"" + re_entry + "\" is not a valid regular expression (" + err.msg + ")",
					file=sys.stderr)
			else:
				print(
					PROGRAM_NAME + ": \"" + re_entry + "\" is not a valid regular expression\n"
					+ "".rjust(len(PROGRAM_NAME + ": \"") + err.colno - 1) + "^ " + err.msg,
					file=sys.stderr)
			exit(1)
	return result


def verify_dir(path):
	if not os.path.exists(path):
		print(PROGRAM_NAME + ": Directory \"" + path + "\" does not exist", file=sys.stderr)
		sys.exit(1)
	if not os.path.isdir(path):
		print(PROGRAM_NAME + ": \"" + path + "\" is not a directory", file=sys.stderr)
		sys.exit(1)


class Settings:
	blacklist = []
	dry_run = False
	executable = ""
	exedir = False
	no_clobber = False
	search_dirs = []
	target_dir = ""
	verbose = False
	whitelist = []

	def __parse__(self):
		parser = argparse.ArgumentParser(prog="copydeps", add_help=False)
		parser.add_argument('EXECUTABLE', type=str, nargs=1)
		parser.add_argument('TARGET-DIR', type=str, nargs="?", default=None)

		parser.add_argument("--blacklist", action="append", metavar="PATTERN")
		parser.add_argument("--dry-run", action="store_true")
		parser.add_argument("--exedir", action="store_true")
		parser.add_argument("--no-clobber", action="store_true")
		parser.add_argument("--search-dir", action="append", metavar="DIRECTORY")
		parser.add_argument("--verbose", action="store_true")
		parser.add_argument("--whitelist", action="append", metavar="PATTERN")

		parser.add_argument("--help", nargs=0, action=HelpAction)
		parser.add_argument(
			"--version", action="version", version=(PROGRAM_NAME + " v." + PROGRAM_VERSION + " by " + PROGRAM_AUTHOR))

		args = parser.parse_args()
		args = vars(args)

		executable = args["EXECUTABLE"][0]
		if not os.path.exists(executable):
			print(PROGRAM_NAME + ": File \"" + executable + "\" does not exist", file=sys.stderr)
			sys.exit(1)
		if not os.path.isfile(executable):
			print(PROGRAM_NAME + ": \"" + executable + "\" is not a regular file", file=sys.stderr)
			sys.exit(1)

		target_dir = args["TARGET-DIR"]
		if target_dir is not None:
			verify_dir(target_dir)
		else:
			target_dir = os.path.dirname(os.path.abspath(executable))
		target_dir = target_dir + "/"

		self.executable = executable
		self.target_dir = target_dir

		self.blacklist = compile_regexes(args["blacklist"])
		self.whitelist = compile_regexes(args["whitelist"])

		self.search_dirs = []
		if args["search_dir"] is not None:
			for dirname in args["search_dir"]:
				dirname = os.path.abspath(dirname) + "/"
				verify_dir(dirname)
				self.search_dirs.append(dirname)

		self.dry_run = args["dry_run"]
		self.exedir = args["exedir"]
		self.no_clobber = args["no_clobber"]
		self.verbose = args["verbose"]


settings = Settings()


def parse_args():
	global settings
	settings.__parse__()

	return settings.executable, settings.target_dir
