#!/usr/bin/python3
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

import os
import sys

from copydeps.version import PROGRAM_AUTHOR, PROGRAM_NAME, PROGRAM_VERSION

executable = None
target_dir = None


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
	global executable, target_dir

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
