copydeps - copy a program's dependencies (.so / .dll files)

copydeps is a small script that can be used to find and copy all .so / .dll
files needed by a program to run. This can be useful when you want to bundle
an application together will all its dependencies.

Usage: copydeps EXECUTABLE [TARGET-DIR]
  EXECUTABLE can be one of the following supported formats:
  - 32-bit ELF
  - 64-bit ELF
  - i386 Microsoft Windows executable
  - x86_64 Microsoft Windows executable
  
  TARGET-DIR specifies the directory to copy the .so / .dll files to.
  When omitted, defaults to the directory of the target executable.

Dependencies:
- cp
- objdump
- python3 >= 3.5
