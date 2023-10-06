#!/usr/bin/env zsh

sudo ddrescue --force "/dev/disk/by-id/$1" "/dev/disk/by-partlabel/$1" "$1"
