#!/usr/bin/env python3
"""Adds to index.json the parameters of each class's unnamed constructor, read from api.json (the same thing
bin/defs.dart writes). For packages generated before it did, and for generators of other languages.

    tools/defs/index_params.py defs/dart/flutter/3.47.7 [more package folders...]
"""
import json, sys

def entries(m):
    return ",\n".join(f"  {json.dumps(k)}: {json.dumps(m[k], separators=(',', ':'), ensure_ascii=False)}" for k in sorted(m))

def run(d):
    api = json.load(open(f"{d}/api.json"))["declarations"]
    idx = json.load(open(f"{d}/index.json"))
    ctors = {x["name"]: c for x in api for c in x.get("constructors", []) if c["name"] == ""}
    for name, cls in idx.get("classes", {}).items():
        cls.pop("params", None)
        if name in ctors:
            cls["params"] = [p["name"] for p in ctors[name].get("params", [])]
    out = ["{", '"format": 1,', f'"libraries": {json.dumps(idx["libraries"], separators=(",", ":"))},',
           f'"reexports": {json.dumps(idx.get("reexports", {}), separators=(",", ":"))},',
           '"symbols": {', entries(idx["symbols"]), "},", '"classes": {', entries(idx.get("classes", {})), "}", "}"]
    open(f"{d}/index.json", "w").write("\n".join(out) + "\n")

for d in sys.argv[1:]:
    run(d)
