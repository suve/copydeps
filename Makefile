#
# Makefile for copydeps
# Copyright (C) 2020 Artur "suve" Iwicki
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

SOURCES := Cargo.toml Cargo.lock $(shell ls src/*.rs)

# -- variables end

.PHONY = all clean debug

all: build/copydeps

debug: build/copydeps-debug

clean:
	rm -rf build/

# -- PHONY targets end

build/copydeps: $(SOURCES)
	mkdir -p build/
	cargo build --release --target-dir=build/
	cp -a build/release/copydeps build/copydeps

build/copydeps-debug: $(SOURCES)
	mkdir -p build/
	cargo build --target-dir=build/
	cp -a build/debug/copydeps build/copydeps-debug
