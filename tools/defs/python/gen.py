#!/usr/bin/env python3
"""Type definitions of Python packages (spec/definitions.md), read from the installed modules with `inspect`.

    tools/python/.venv/bin/python tools/defs/python/gen.py <name> <version> <out-root> <module>... \
        [--depends a@1,b@2] [--implicit builtins] [--description "..."]

Writes <out-root>/python/<name>/<version>/{package.toml,index.json,api.json}. Types are written in NuclearLang
notation (str → string, list[X] → X[], dict[K, V] → <K, V>, X | None → X?, Callable[[A], R] → (a0: A) => R).
A name is exported by the first module of the list that has it; only objects defined inside the package (or
re-exported by it) are kept, so `typing.Any` imported by a module does not count as one of its names.
"""
import argparse
import builtins
import enum
import importlib
import inspect
import json
import os
import re
import types
import typing
import warnings

warnings.filterwarnings("ignore")
BASIC = {"str": "string", "int": "int", "float": "double", "bool": "boolean", "None": "void", "NoneType": "void",
         "Any": "any", "object": "object", "list": "list", "dict": "dict", "bytes": "bytes", "complex": "complex"}


def short(name):
    return name.rsplit(".", 1)[-1]


def ty(t, depth=0):
    """A Python annotation in NuclearLang notation."""
    try:
        r = _ty(t, depth)
    except Exception:
        return "any"
    return r if isinstance(r, str) and r else "any"


def _ty(t, depth=0):
    if depth > 6:
        return "any"
    if t is inspect.Parameter.empty or t is inspect.Signature.empty:
        return "any"
    if t is None or t is type(None):
        return "void"
    if isinstance(t, str):
        return text_type(t, depth)
    if isinstance(t, typing.TypeVar):
        return t.__name__
    origin = typing.get_origin(t)
    args = [a for a in typing.get_args(t)]
    if origin in (typing.Union, types.UnionType):
        rest = [a for a in args if a is not type(None)]
        inner = " | ".join(ty(a, depth + 1) for a in rest) or "any"
        if len(rest) > 1 and len(rest) < len(args):
            inner = f"({inner})"
        return inner + ("?" if len(rest) < len(args) else "")
    if origin in (list, typing.List, set, frozenset, tuple) or (origin and getattr(origin, "__name__", "") in ("Sequence", "Iterable", "MutableSequence")):
        name = getattr(origin, "__name__", "list")
        if origin in (list, typing.List):
            return f"{ty(args[0], depth + 1) if args else 'any'}[]"
        return f"{name}<{', '.join(ty(a, depth + 1) for a in args if a is not Ellipsis)}>" if args else name
    if origin in (dict, typing.Dict):
        return f"<{ty(args[0], depth + 1)}, {ty(args[1], depth + 1)}>" if len(args) == 2 else "<any, any>"
    if origin is typing.Literal:
        return " | ".join(json.dumps(a) if isinstance(a, str) else str(a) for a in args) or "any"
    if origin is typing.Annotated:
        return ty(args[0], depth + 1)
    if origin and getattr(origin, "__name__", "") == "Callable" or origin is getattr(__import__("collections.abc").abc, "Callable"):
        if args and isinstance(args[0], list):
            ps = ", ".join(f"a{i}: {ty(a, depth + 1)}" for i, a in enumerate(args[0]))
            return f"({ps}) => {ty(args[1], depth + 1)}"
        return "(...args: any) => any"
    if origin is not None:
        name = getattr(origin, "__name__", None) or short(str(origin))
        return f"{BASIC.get(name, name)}<{', '.join(ty(a, depth + 1) for a in args)}>" if args else BASIC.get(name, name)
    if isinstance(t, type):
        return BASIC.get(t.__name__, t.__name__)
    name = getattr(t, "__name__", None)
    return BASIC.get(name, name) if name else "any"


def split_top(s, sep):
    """Splits `s` at `sep` outside brackets."""
    parts, depth, cur = [], 0, ""
    for ch in s:
        if ch in "[(":
            depth += 1
        elif ch in "])":
            depth -= 1
        if ch == sep and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    parts.append(cur)
    return [p.strip() for p in parts]


def text_type(s, depth=0):
    """A string annotation (`from __future__ import annotations`, or one get_type_hints cannot resolve)."""
    s = s.strip().strip("'\"")
    if depth > 8 or not s:
        return "any"
    alts = split_top(s, "|")
    if len(alts) > 1:
        rest = [a for a in alts if a not in ("None", "NoneType")]
        inner = " | ".join(text_type(a, depth + 1) for a in rest) or "any"
        if len(rest) > 1 and len(rest) < len(alts):
            inner = f"({inner})"
        return inner + ("?" if len(rest) < len(alts) else "")
    m = re.fullmatch(r"([\w.]+)\[(.*)\]", s, re.S)
    if not m:
        base = short(s) if re.fullmatch(r"[\w.]+", s) else "any"
        return BASIC.get(base, base)
    base, args = short(m[1]), split_top(m[2], ",")
    if base == "Optional":
        return text_type(args[0], depth + 1) + "?"
    if base == "Union":
        return text_type(" | ".join(args), depth + 1)
    if base in ("list", "List", "Sequence", "Iterable") and base in ("list", "List"):
        return f"{text_type(args[0], depth + 1)}[]"
    if base in ("dict", "Dict", "Mapping") and len(args) == 2 and base in ("dict", "Dict"):
        return f"<{text_type(args[0], depth + 1)}, {text_type(args[1], depth + 1)}>"
    if base == "Callable" and args and args[0].startswith("["):
        ps = [a for a in split_top(args[0][1:-1], ",") if a]
        rendered = ", ".join(f"a{i}: {text_type(a, depth + 1)}" for i, a in enumerate(ps))
        return f"({rendered}) => {text_type(args[1], depth + 1) if len(args) > 1 else 'any'}"
    if base in ("Annotated", "ClassVar", "Final", "Required", "NotRequired"):
        return text_type(args[0], depth + 1)
    if base == "Literal":
        return " | ".join(args)
    return f"{BASIC.get(base, base)}<{', '.join(text_type(a, depth + 1) for a in args if a != '...')}>"


def hints(obj):
    try:
        return typing.get_type_hints(obj, include_extras=False)
    except Exception:
        return getattr(obj, "__annotations__", {}) or {}


def params(fn, skip_self):
    try:
        sig = inspect.signature(fn)
    except (TypeError, ValueError):
        return None, None
    h = hints(fn)
    out, open_kwargs = [], False
    for i, p in enumerate(sig.parameters.values()):
        if skip_self and i == 0 and p.name in ("self", "cls"):
            continue
        if p.kind == p.VAR_KEYWORD:
            open_kwargs = True
            continue
        if p.kind == p.VAR_POSITIONAL:
            out.append({"name": "..." + p.name, "type": "any[]", "optional": True})
            continue
        item = {"name": p.name, "type": ty(h.get(p.name, p.annotation))}
        if p.kind == p.KEYWORD_ONLY:
            item["named"] = True
        if p.default is not p.empty:
            item["optional"] = True
            d = p.default
            if d is None or isinstance(d, (bool, int, float, str)) and len(repr(d)) < 40:
                item["default"] = repr(d)
        out.append(item)
    ret = h.get("return", sig.return_annotation)
    return (out, open_kwargs), ty(ret)


def doc(obj):
    d = inspect.getdoc(obj) or ""
    first = d.strip().split("\n\n")[0].replace("\n", " ").strip()
    return first[:200] or None


def member_list(cls):
    out = []
    own_hints = hints(cls)
    seen = set()
    for name, v in sorted(vars(cls).items()):
        if name.startswith("_") or name in seen:
            continue
        seen.add(name)
        m = {"name": name}
        raw = v
        if isinstance(v, (staticmethod, classmethod)):
            m["static"] = True
            raw = v.__func__
        if isinstance(v, property):
            m["kind"] = "property"
            r = hints(v.fget).get("return") if v.fget else None
            m["type"] = ty(r) if r is not None else "any"
            if v.fset is None:
                m["readonly"] = True
        elif callable(raw) and (inspect.isfunction(raw) or inspect.ismethoddescriptor(raw) or inspect.isbuiltin(raw)):
            m["kind"] = "method"
            ps, r = params(raw, not m.get("static") or isinstance(v, classmethod))
            if ps:
                m["params"] = ps[0]
            if r:
                m["returns"] = r
        else:
            m["kind"] = "property"
            m["type"] = ty(own_hints.get(name, type(v)))
        d = doc(raw) if m["kind"] == "method" else None
        if d:
            m["doc"] = d
        out.append(m)
    for name, h in own_hints.items():
        if not name.startswith("_") and name not in seen and name != "model_config":
            seen.add(name)
            out.append({"name": name, "kind": "property", "type": ty(h)})
    return out


def declaration(name, obj, lib):
    d = {"name": name, "library": lib}
    if inspect.isclass(obj):
        bases = [b.__name__ for b in obj.__bases__ if b is not object]
        if issubclass(obj, enum.Enum):
            d["kind"] = "enum"
            d["values"] = [m.name for m in obj]
        else:
            d["kind"] = "class"
        if bases:
            d["extends"] = bases[0]
            if bases[1:]:
                d["with"] = bases[1:]
        tp = getattr(obj, "__type_params__", ()) or getattr(obj, "__parameters__", ())
        if tp:
            d["type_params"] = [t.__name__ for t in tp if hasattr(t, "__name__")]
        if inspect.isabstract(obj):
            d["modifiers"] = ["abstract"]
        if d["kind"] == "class":
            ps, _ = params(obj, False)
            if ps is not None:
                c = {"name": "", "params": ps[0]}
                if ps[1]:
                    c["open"] = True
                d["constructors"] = [c]
        d["members"] = member_list(obj)
    elif inspect.isfunction(obj) or inspect.isbuiltin(obj) or callable(obj) and not isinstance(obj, type):
        d["kind"] = "function"
        ps, r = params(obj, False)
        if ps:
            d["params"] = ps[0]
        if r:
            d["returns"] = r
    else:
        d["kind"] = "variable"
        d["type"] = ty(type(obj))
    text = doc(obj) if d["kind"] != "variable" else None
    if text:
        d["doc"] = text
    return d


def public_names(mod):
    names = getattr(mod, "__all__", None)
    if names is None:
        names = [n for n in dir(mod) if not n.startswith("_")]
    return [n for n in names if isinstance(n, str) and not n.startswith("_")]


def main():
    a = argparse.ArgumentParser()
    a.add_argument("name")
    a.add_argument("version")
    a.add_argument("out")
    a.add_argument("modules", nargs="+")
    a.add_argument("--depends", default="")
    a.add_argument("--implicit", default="")
    a.add_argument("--description", default="")
    a.add_argument("--roots", default="", help="module prefixes whose objects count (default: the modules' tops)")
    o = a.parse_args()
    roots = tuple(o.roots.split(",")) if o.roots else tuple({m.split(".")[0] for m in o.modules})
    implicit = [m for m in o.implicit.split(",") if m]
    decls, symbols, classes = {}, {}, {}
    for li, modname in enumerate(o.modules):
        mod = importlib.import_module(modname)
        for n in public_names(mod):
            if n in decls:
                continue
            try:
                obj = getattr(mod, n)
            except Exception:
                continue
            if inspect.ismodule(obj):
                continue
            home = getattr(obj, "__module__", None) or modname
            if modname != "builtins" and not (isinstance(home, str) and home.startswith(roots)):
                if inspect.isclass(obj) or inspect.isfunction(obj):
                    continue
            try:
                d = declaration(n, obj, modname)
            except Exception:
                continue
            decls[n] = d
            symbols[n] = li
            if d["kind"] == "class":
                ctor = (d.get("constructors") or [None])[0]
                entry = {"supers": [d["extends"]] + d.get("with", []) if "extends" in d else [],
                         "members": sorted(m["name"] for m in d.get("members", []) if not m.get("static"))}
                # methods that return Nuclear's Result (the transpiler unwraps them without reading api.json)
                results = {m["name"]: m["returns"] for m in d.get("members", [])
                           if m.get("kind") == "method" and str(m.get("returns", "")).startswith("Result<")}
                if results:
                    entry["results"] = results
                if ctor is not None and not ctor.get("open"):
                    entry["params"] = [p["name"] for p in ctor["params"] if not p["name"].startswith("...")]
                classes[n] = entry
    out = os.path.join(o.out, "python", o.name, o.version)
    os.makedirs(out, exist_ok=True)
    dep = [d for d in o.depends.split(",") if d]
    with open(os.path.join(out, "package.toml"), "w") as f:
        f.write("# Generated by tools/defs/python/gen.py. Do not edit by hand: regenerate it (tools/defs/python/generate.sh).\n")
        f.write(f'[package]\nname = "{o.name}"\nversion = "{o.version}"\nlanguage = "python"\n')
        f.write(f"description = {json.dumps(o.description or o.name)}\n")
        f.write(f"depends = [{', '.join(json.dumps(x) for x in dep)}]\nlibraries = [\n")
        f.writelines(f'  "{m}",\n' for m in o.modules)
        f.write("]\n")
        if implicit:
            f.write(f"implicit = [{', '.join(json.dumps(x) for x in implicit)}]\n")
    enc = lambda v: json.dumps(v, separators=(",", ":"), ensure_ascii=False)
    entries = lambda m: ",\n".join(f"  {json.dumps(k)}: {enc(m[k])}" for k in sorted(m))
    with open(os.path.join(out, "index.json"), "w") as f:
        f.write("{\n\"format\": 1,\n")
        f.write(f'"libraries": {enc(o.modules)},\n"reexports": {{}},\n')
        f.write('"symbols": {\n' + entries(symbols) + "\n},\n")
        f.write('"classes": {\n' + entries(classes) + "\n}\n}\n")
    with open(os.path.join(out, "api.json"), "w") as f:
        f.write('{"format": 1, "declarations": [\n')
        f.write(",\n".join(enc(decls[k]) for k in sorted(decls)))
        f.write("\n]}\n")
    print(f"{out}: {len(symbols)} symbols, {len(classes)} classes")


if __name__ == "__main__":
    main()
