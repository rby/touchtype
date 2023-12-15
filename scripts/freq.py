#! /usr/bin/env nix-shell
#! nix-shell -i python3 -p python3

from collections import Counter
import sys
import re


frequencies =  Counter()
while line := sys.stdin.readline():
    for w in re.findall(r"\w+", line.strip()):
        frequencies[w.lower()] += 1

for w, c in frequencies.items():
    if len(w) >= 3:
        print(f"{w} {c}")
