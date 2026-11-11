---
name: nuclear
description: Write, fix or review NuclearLang code (.nuc files, nuclear.toml, Dragonfly projects with config.nuc: mobile apps with the flutter dialect, backends with the fastapi (Python) or nestjs (TypeScript) dialect). Use it whenever a task touches a .nuc file or the nuclear / dragonfly command line tools.
---

# NuclearLang

NuclearLang (`.nuc`) is a minimal language transpiled to TypeScript, Dart or Python by declarative dialects. Dragonfly
is its framework: three platforms written in Nuclear — `mobile` (dialect `flutter`: the Flutter app is generated in
`build/app`), `backend` (dialects `fastapi` = Python/FastAPI and `nestjs` = TypeScript, built to `build/` by
`nuclear build .`) and `web` (no dialect yet). The Nuclear of a platform is the same for every dialect of it; only
the standard library of Nuclear is used on strings, lists and maps (`spec/library.md`), never the target's. Its
syntax is NOT TypeScript, Dart nor Python: read the sheet before writing.

## 1. Load the sheet (once per task, ~3k tokens)

```sh
nuclear spec --brief --dialect flutter     # Dragonfly app (config.nuc + main.nuc, no nuclear.toml)
nuclear spec --brief --dialect <dialect>   # the dialect of config.nuc (fastapi, nestjs) or nuclear.toml (typescript, dart, python)
```

It says only what changes from TypeScript/Dart. Do not load the full spec (`nuclear spec`) unless the sheet and
`nuclear help <topic>` do not answer.

## 2. Work cycle

1. Dragonfly: create each piece with its generator, so it lands in the right folder with the right kind:
   `dragonfly make component|screen|usecase|repository|model|event|guard|widget|page|test ...` (`dragonfly help`).
2. Write the code following the sheet. Copy the style of the project's existing `.nuc` files.
3. Check until clean, and fix exactly what each error says (most carry a `fix`):
   - Dragonfly: `dragonfly check --json` (must print `[]`), then `dragonfly analyze` (no `error •` lines) and, with
     tests, `dragonfly test`.
   - fastapi: `nuclear check . --json`, `nuclear build .`, then in `build/`: `pyright` and `pytest`.
   - Other projects: `nuclear check <folder> --json`; a single file: `nuclear check file.nuc --dialect <d> --json`,
     then `nuclear build file.nuc --dialect <d> -o file.ts`.
4. Format what you wrote: `nuclear fmt <folder>` (only spacing and indentation; `--check` lists what would change).
5. Never edit generated files (`build/`, `*.g.dart`, the `.ts`/`.dart` output): change the `.nuc` and rebuild.

## 3. Look up instead of guessing

- A class, constructor, argument or member of Flutter, Dart or a package:
  `nuclear defs show <Name> --dialect flutter` (or `<Name.member>`); `nuclear defs search <text> --dialect flutter`.
- A kind, decorator or header word of the dialect (what it generates, its rules, an example):
  `nuclear help <word> --dialect <d>` (`repository`, `@screen`...); every word: `nuclear help words --dialect <d>`.
- A language or framework topic: `nuclear help` lists them (`results`, `flutter`, `dragonfly`, `tutorial`...).
- A runtime error of the app: `dragonfly explain` (reads the error on stdin and points to the `.nuc` line).

## 4. Mistakes agents make

The compiler catches the habits of TypeScript, Dart and Python (`N0022`, with an exact `fix`), invented classes
(`N0220`) and invented named arguments (`N0219`, it lists the real ones). Apply the fix instead of rewriting.

- Writing `return`, `;`, `{ }`, `new`, `await`, `const`/`final`/`var`, `&&`, `!x`, `name: value` arguments: use the
  sheet's forms (`~x`, `x = 1`, `and`, `not x`, `name = value`).
- The return type goes after the parameters: `load(id: int) User`.
- In Dragonfly, a screen calls its `statemanager`, which calls the component's `appcomponent`; only the use case
  calls the repository. Another component sees only the `appcomponent`, models, events, guards and screens.
- Texts shown to the user are translation keys: `"component.screen.key".tr()` plus the key in
  `assets/i18n/<locale>/<component>.json`.
