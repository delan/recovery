#!/usr/bin/env zsh
set -eux

size=$(./get-size "$1")
printf 'n\n\n\n+%s\nx\nn\n\n%s\np\nr\np\nw\n' $((size-1)) "$1" | sudo fdisk "/dev/disk/by-id/$2"
