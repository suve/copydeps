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
import sys

from copydeps.get_deps import get_deps
from copydeps.run_program import run
from copydeps.settings import parse_args
from copydeps.version import PROGRAM_NAME


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


def main():
	executable, target_dir = parse_args()
	deps = get_deps(executable)
	copy_deps(deps, target_dir)


if __name__ == "__main__":
	main()
