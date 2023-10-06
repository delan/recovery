#!/usr/bin/env python3

import sys

with open(sys.argv[1], "rb") as f:
	f.seek(0, 2)
	sectors = f.tell() // 512
	for i in range(sectors):
		if i % 2048 == 0:
			print(f"{i // 2048} MiB", end="\r")
		f.seek(i * 512)
		if f.read(3) == b"\xeb\x52\x90":
			f.seek(i * 512 + 510)
			if f.read(2) == b"\x55\xaa":
				print(f"found {i}")
