copydeps - copy a program's dependencies (.so / .dll files)

copydeps is a small program that can be used to find and copy all .so / .dll
files needed by a program to run. This can be useful when you want to bundle
an application together will all its dependencies.

Usage: copydeps [options...] EXECUTABLE [TARGET-DIR]
  EXECUTABLE can be one of the following supported formats:
  - 32-bit ELF
  - 64-bit ELF
  - i386 Microsoft Windows executable
  - x86_64 Microsoft Windows executable
  
  TARGET-DIR specifies the directory to copy the .so / .dll files to.
  When omitted, defaults to the directory of the target executable.

Program options:
--blacklist PATTERN
  Add PATTERN to the built-in blacklist (.so / .dll names that should not
  be resolved nor copied over).
--dry-run
  Print the list of dependencies without actually copying the .so / .dll files.
--exedir
  Include the directory of the executable in the .so / .dll resolve paths.
  Files found in the exedir are preferred over those in system paths.
--no-clobber
  Do not overwrite .so / .dll files already existing in the target directory.
--verbose
  Print the names of the dependencies as they're being copied over.


Dependencies:
- cp
- objdump
- python3 >= 3.5
