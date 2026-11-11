<!-- nuclearlang -->
## NuclearLang

This project is written in NuclearLang (`.nuc`), dialect `{dialect}`. Its syntax is not TypeScript, Dart nor Python.

- Before writing code, read the sheet: `nuclear spec --brief --dialect {dialect}`.
- After each change: `{check}` must report no errors; fix exactly what each error says (most carry a `fix`).
- Look up instead of guessing: an API `nuclear defs show <Name> --dialect {dialect}`; a kind or decorator
  `nuclear help <word> --dialect {dialect}` (all: `nuclear help words --dialect {dialect}`); a topic `nuclear help <topic>`.
{dragonfly}- Never edit generated code ({generated}): change the `.nuc` files.
<!-- /nuclearlang -->
