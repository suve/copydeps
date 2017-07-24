#!/bin/bash

blacklist=("libasan." "libc." "libgcc_s." "libm." "libpthread." "libstdc++.")

if [ "$#" -lt 1 ]; then
	echo "cpso: EXECUTABLE is missing"
	echo "Usage: cpso EXECUTABLE [target-dir]"
	exit
fi

if [ ! -e "$1" ]; then
	if [ "$1" == "--help" ]; then
		echo "cpso is a script for bundling the .so files needed by binary executables."
		echo "Usage: cpso EXECUTABLE [target-dir]"
	else
		echo "cpso: File '$1' does not exist"
	fi
	exit
fi

if [ "$#" -ge 2 ]; then
	if [ ! -d "$2" ]; then
		echo "cpso: Directory '$2' does not exist"
		exit
	fi
	
	dir=$2
else
	dir=`pwd`
fi

OLD_IFS=$IFS
IFS="
"

so_list=`ldd "$1" | grep '=>' | sed -e 's/^[ \t]*//' -e 's/(0x[0-9a-f]*)//'`
so_array=($so_list)

IFS=$OLD_IFS

for ((i=0; i<${#so_array[@]}; ++i)); do
	so_entry=${so_array[$i]}
	
	so_source=`echo "$so_entry" | sed -e 's/.* => //' | sed -e 's/^[ \t]*//' -e 's/[ \t]*$//'`
	so_target=`echo "$so_entry" | sed -e 's/ => .*//' | sed -e 's/^[ \t]*//' -e 's/[ \t]*$//'`
	
	do_not_copy=0
	for ((b=0; b<${#blacklist[@]}; ++b)); do
		blackentry=${blacklist[$b]}
		
		echo "$so_target" | grep -F -e "$blackentry" -q -
		if [ "$?" -eq 0 ]; then
			do_not_copy=1
			b=${#blacklist[@]}
			break
		fi
	done
	
	if [ $do_not_copy -eq 0 ]; then
		error=`cp --preserve=timestamps "$so_source" "$dir/$so_target" 2>/dev/fd/1`
		
		if [ "$?" -eq 0 ]; then
			echo "cpso: '$so_target' copied from '$so_source'"
		else
			echo "cpso: '$so_target' could not be copied ($error)"
		fi
	else
		echo "cpso: '$so_target' is blacklisted, skipping"
	fi
done
