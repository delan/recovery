#!/usr/bin/env zsh

sudo fdisk -l "/dev/disk/by-id/$1" | sed q | egrep -o '[0-9]+ sectors$' | cut -d' ' -f1
