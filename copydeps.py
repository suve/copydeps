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
from copydeps.settings import settings, parse_args
from copydeps.version import PROGRAM_NAME


def print_deps(deplist):
	for dep in deplist:
		if dep.isBlacklisted:
			print("\"" + dep.name + "\" -> (blacklisted)")
		elif dep.path is None:
			print("\"" + dep.name + "\" -> (unable to resolve)")
		else:
			print("\"" + dep.name + "\" -> \"" + dep.path + "\"")


def copy_deps(deplist, target_dir):
	for dep in deplist:
		if dep.isBlacklisted:
			print(PROGRAM_NAME + ": \"" + dep.name + "\" is blacklisted, skipping")
			continue

		if dep.path is None:
			print(PROGRAM_NAME + ": unable to resolve \"" + dep.name + "\"")
			continue

		code, _, err = run("cp", ["--preserve=timestamps", dep.path, target_dir])
		if code == 0:
			print(PROGRAM_NAME + ": \"" + dep.name + "\" copied from \"" + dep.path + "\"")
		else:
			print(PROGRAM_NAME + ": \"" + dep.name + "\" could not be copied (" + err[0] + ")")


def main():
	executable, target_dir = parse_args()
	deps = get_deps(executable)

	if settings.dry_run:
		print_deps(deps)
	else:
		copy_deps(deps, target_dir)

	# Exit successfully only if all dependencies were resolved and copied
	for dep in deps:
		if (not dep.isBlacklisted) and (dep.path is None):
			sys.exit(1)


if __name__ == "__main__":
	main()
