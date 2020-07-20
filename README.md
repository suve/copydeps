# copydeps

**copydeps** finds and copies all .so / .dll files needed by a program to run.
This can be useful when you want to bundle an application
together will all of its dependencies.

## Usage

```
copydeps [options...] EXECUTABLE [TARGET-DIR]
```

*EXECUTABLE* can be one of the following supported formats:
- 32-bit ELF
- 64-bit ELF
- i386 Microsoft Windows executable
- x86_64 Microsoft Windows executable

*TARGET-DIR* specifies the directory to copy the .so / .dll files to.
When omitted, defaults to the directory of the target executable.

### Program options

- `--dry-run`  
  Print the list of dependencies without actually copying the .so / .dll files.
- `--exedir`  
  Include the directory of the executable in the .so / .dll resolve paths.
  Files found in the exedir are preferred over those found anywhere else.
- `--ignore PATTERN`  
  Add the regular expression *PATTERN* to the ignore-list
  (.so / .dll names that should not be resolved nor copied over).
- `--no-clobber`  
  Do not overwrite .so / .dll files already existing in the target directory.
- `--override PATTERN`  
  Add the regular expression *PATTERN* to the override-list
  (.so / .dll names that should always be resolved and copied over).
  Overrides have precedence over ignores.
- `--search-dir DIRECTORY`  
  Add *DIRECTORY* to the list of paths to search when resolving .so / .dll names.
  User-specified directories take precedence over system paths.
- `--verbose`  
  Print the names of the dependencies as they're being copied over.

## Building from source

**copydeps** is written in Rust and uses Cargo for keeping track of its dependencies.
While you may invoke `cargo` directly, it's recommended to use `make` instead.

```
cd copydeps/
make -j all
[sudo] make install
```

## Licence
**copydeps** is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License, either version 3 of the License, or (at your option) any later version.

For the full text of the licence, consult [LICENCE.txt](blob/trunk/LICENCE.txt).
