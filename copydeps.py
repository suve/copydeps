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
			print("\"" + dep.name + "\": (blacklisted)")
		elif dep.path is None:
			print("\"" + dep.name + "\": (unable to resolve)")
		else:
			print("\"" + dep.name + "\": " + dep.path)
	return True


def copy_deps(deplist, target_dir):
	all_ok = True
	for dep in deplist:
		if dep.isBlacklisted:
			if settings.verbose:
				print("\"" + dep.name + "\": blacklisted, skipping")
			continue

		if dep.path is None:
			print(PROGRAM_NAME + ": unable to resolve \"" + dep.name + "\"", file=sys.stderr)
			all_ok = False
			continue

		target_path = os.path.abspath(target_dir + "/" + dep.name)
		if os.path.samefile(dep.path, target_path):
			if settings.verbose:
				print("\"" + dep.name + "\": preferred version already present in the target directory")
			continue

		code, _, err = run("cp", ["--preserve=timestamps", dep.path, target_path])
		if code == 0:
			if settings.verbose:
				print("\"" + dep.name + "\": " + dep.path + " -> " + target_path)
		else:
			print(PROGRAM_NAME + ": \"" + dep.name + "\" could not be copied (" + err[0] + ")", file=sys.stderr)
			all_ok = False

	return all_ok


def main():
	executable, target_dir = parse_args()
	deps = get_deps(executable)

	if settings.dry_run:
		success = print_deps(deps)
	else:
		success = copy_deps(deps, target_dir)

	sys.exit(0 if success else 1)


if __name__ == "__main__":
	main()
