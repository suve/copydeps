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

from copydeps.version import PROGRAM_VERSION
from setuptools import setup, find_packages


with open("README.txt", "r") as fh:
	long_description = fh.read()

setup(
	name="copydeps",
	version=PROGRAM_VERSION,
	author="suve",
	author_email="veg@svgames.pl",
	description="Find and copy .so / .dll files required by a binary executable",
	long_description=long_description,
	long_description_content_type="text/plain",
	url="https://github.com/suve/copydeps/",
	license="GNU General Public License v3 or later (GPLv3+)",
	license_file="LICENCE.txt",
	packages=find_packages(),
	entry_points={
		"console_scripts": [
			"copydeps = copydeps.copydeps:copydeps_main"
		]
	},
	classifiers=[
		"Development Status :: 5 - Production/Stable",
		"Environment :: Console",
		"Intended Audience :: Developers",
		"License :: OSI Approved :: GNU General Public License v3 or later (GPLv3+)",
		"Natural Language :: English",
		"Operating System :: POSIX",
		"Programming Language :: Python :: 3.5",
		"Topic :: Software Development",
	]
)
