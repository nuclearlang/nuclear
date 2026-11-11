# Creating a dialect with an agent

This tutorial shows how to create a new NuclearLang dialect (a language like `crystal` or a framework like
`amber`) **using a coding agent**: Claude Code, Codex or OpenCode. The dialect is not written by hand: you
describe what you want and review; the agent writes the files and checks them with the transpiler.

The rules the agent follows are in [spec/dialects.md §8](../../spec/dialects.md) (purpose, what it can do,
process, **restrictions** and when it is done). The agent reads them with `nuclear spec`.

## 1. Before you start

- NuclearLang installed (`./install.sh`): `nuclear help` must answer.
- The target language's compiler or SDK, to test that the generated code compiles (for example `crystal` for
  Crystal).
- A working folder. The agent creates the dialect in `dialects/<name>/` inside it; the transpiler looks there by
  itself.

```sh
mkdir crystal-dialect && cd crystal-dialect
```

## 2. What to ask the agent

Tell it the *what for*, not the *how*: which language or framework, which version, and what you want Nuclear to
save you. The agent decides the rest following §8.

### A language (target dialect)

```
Create the NuclearLang dialect `crystal` for Crystal 1.14.
Follow spec/dialects.md §8 to the letter: read the contract with `nuclear spec`, use
dialects/typescript as the structure reference, write dialects/crystal/spec.md first with one
example per rule, then dialect.toml, keywords.toml and the templates.
Validate with `nuclear dialect check crystal --json` until there are no errors, compile each example with
`nuclear build <file.nuc> --dialect crystal` and compile it with `crystal build --no-codegen`.
Respect the restrictions of §8.4: if something of the base language cannot be written with templates,
stop and tell me, do not fake it. At the end give me the list of covered constructs and the missing ones.
```

### A framework (a dialect that extends another)

```
Create the NuclearLang dialect `amber`, which extends `crystal`, for Amber 1.4.
Follow spec/dialects.md §8. Read `nuclear spec --dialect crystal` and use dialects/backend/nestjs as the
reference for backend kinds (controller, service). Each kind must remove repeated Amber code,
with an example in dialects/amber/spec.md. Validate with `nuclear dialect check amber --json`.
```

### A Dragonfly framework

If you want it to be Dragonfly (components, layers, use cases, repositories...), say so:

```
Create `amber` as a Dragonfly dialect: apply to Amber the Dragonfly rules of spec/flutter.md
§1, §3, §4 and §6 (components, layers, architecture kinds), written as an Amber expert would
write them. Do not copy anything from Flutter that makes no sense in a backend; ask me when in doubt.
```

## 3. What the agent does (so you know how to review it)

1. Reads `nuclear spec` (the base language, the dialect format and the list of templates, variables and
   options).
2. Writes `dialects/<name>/spec.md`: each rule with a `nuclear` block and the target code that must come out.
3. Writes `dialect.toml`, the templates and `keywords.toml` until `nuclear dialect check` gives `[]` (no errors).
4. Compiles each example with `nuclear build` and fixes the templates until the output is that of `spec.md`.
5. Compiles the output with the target's compiler.
6. Delivers the coverage and what it could not do.

A real example of what a minimal kind produces (`handler`, on top of `typescript`):

```toml
# dialects/minest/dialect.toml
[dialect]
name = "minest"
extends = "typescript"

[kinds.handler]
description = "handler Name + methods → exported class"
template = "export class {{name}}Handler {\n  {{members}}\n}"
```

```nuclear
handler Users
  list() int[]
    [1, 2]
```

```typescript
export class UsersHandler {
  list(): number[] {
    return [1, 2];
  }
}
```

## 4. How to review the result

| Check | Command | It must give |
|---|---|---|
| The dialect is valid | `nuclear dialect check <name> --json` | `[]` |
| A file compiles | `nuclear check file.nuc --dialect <name> --json` | `[]` |
| The output | `nuclear build file.nuc --dialect <name>` | the code of `spec.md` |
| The spec other agents will see | `nuclear spec --dialect <name>` | the base + the specs of the chain |

And read `spec.md`: each rule must have its example, without questions nor pending items.

## 5. What the agent cannot do

The full restrictions are in [spec/dialects.md §8.4](../../spec/dialects.md). The ones that matter most when
reviewing:

- **It does not touch the transpiler nor Nuclear's syntax.** If the target needs something that does not come
  out of a template, the agent tells you; that change is made by the NuclearLang team.
- **It does not edit the shipped dialects** (`typescript`, `dart`, `nodejs`, `nestjs`, `flutter`): it extends
  them.
- **It does not invent a second way of writing the same thing**: each kind removes repeated code.
- **It leaves no rule without an example** nor examples that are not the real output.
- **It writes everything in English**: the TOML comments, the templates, the examples and `spec.md`.
- **It does not generate code an expert in the target would not write** (reflection, unused layers, compiler
  warnings).

If you see any of this in what it delivered, ask it to fix it quoting the restriction.

## 6. Using it

```sh
nuclear build app.nuc --dialect crystal -o app.cr
```

To use it from another folder: `--dialect-path <folder>/dialects` or the `NUCLEAR_DIALECT_PATH` variable.

What comes next (`specs/002-dialects-with-agents/plan.md`): `nuclear dialect new` and `nuclear dialect test`, a
skill that loads these rules by itself in Claude Code and an `AGENTS.md` for Codex and OpenCode, and dialects
installable from a repository with `dragonfly <dialect> new`.
