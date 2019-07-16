#!/usr/bin/python3
# -*- coding: utf-8 -*-
#
# cpso.py - copy program's dependencies (.so files)
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


def run(program, args = []):
	args.insert(0, program)
	proc = subprocess.run(args=args, capture_output=True, text=True)
	return proc.stdout.split("\n")


def get_deps(executable):
	output = run("ldd", [executable])

	deps = {}
	for line in output:
		if not " => " in line:
			continue
		
		parts = line.split(" => ")

		name = parts[0].strip()
		target = parts[1].split(" ")[0]

		deps[name] = target

	return deps


def copy_deps(deps, target_dir):
	blacklist = ["libasan.", "libc.", "libgcc_s.", "libm.", "libpthread.", "libstdc++."]

	for key, value in deps.items():
		so_name = key
		so_path = value

		copy = True
		for blackentry in blacklist:
			if blackentry in so_name:
				copy = False
				break

		if copy:
			run("cp", ["--preserve=timestamps", so_path, target_dir])
			print("cpso: \"" + so_name + "\" copied from \"" + so_path + "\"")
		else:
			print("cpso: \"" + so_name + "\" is blacklisted, skipping")


def parse_args():
	argc = len(sys.argv)
	if argc < 2:
		print("cpso: EXECUTABLE is missing\nUsage: cpso EXECUTABLE [TARGET-DIR]", file=sys.stderr)
		sys.exit(1)

	if sys.argv[1] == "--help":
		print("cpso is a script for bundling the .so files needed by binary executables.\nUsage: cpso EXECUTABLE [TARGET-DIR]", file=sys.stderr)
		sys.exit(0)

	executable = sys.argv[1]
	if not os.path.isfile(executable):
		print("cpso: File \"" + executable + "\" does not exist", file=sys.stderr)
		sys.exit(1)

	if argc >= 3:
		target_dir = sys.argv[2]
		if not os.path.isdir(target_dir):
			print("cpso: Directory \"" + target_dir + "\" does not exist", file=sys.stderr)
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
