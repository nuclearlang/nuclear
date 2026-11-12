# Multi-file projects

A **project** is a folder with a `nuclear.toml` manifest and `.nuc` files in any subfolder. It is
transpiled as a whole: each file sees the declarations of the others. This document describes the
mechanism, which is the same for every dialect; what each framework does with it is in its spec (for
example, the [flutter](https://github.com/nuclearlang/dragonfly-mobile/blob/main/spec/flutter.md) dialect).

## 1. Manifest

A Dragonfly project (a platform of ADR-0093) has only `.nuc` files; its manifest is `config.nuc`:

```nuclear
// config.nuc
dragonfly(platform = "backend", dialect = "fastapi")   // the platform written, the dialect it is built with
app(name = "tasks_api", description = "Tasks and an assistant.", version = "0.1.0")
plugins(sqlite = true, ollama = true)                   // §3.3
```

`dragonfly(...)` is required: `platform` is `web`, `mobile` or `backend`; `dialect` one of the platform's dialects
(N0303 otherwise). `app(...)` names the generated package (`name`, in snake_case) and its description and
version; `title` is the app's name as people read it (`title = "NuclearLang"`: the window or tab title; the name
when absent); a platform may take more fields (`usesMaterialDesign` in mobile). The output folder is the dialect's
(`[project] output`, §3). Changing `dialect` changes nothing else in the project.

A project of a language dialect without Dragonfly uses `nuclear.toml`:

```toml
# nuclear.toml
[project]
name = "rick_demo"      # name of the generated package, in snake_case
dialect = "python"      # a language dialect: typescript, dart, python
out = "build"           # output folder, relative to the project (default: build)

[dependencies]          # packages of the target's package manager the project uses
httpx = "0.28"
```

```sh
nuclear build <folder>               # transpiles the project into <folder>/<out>
nuclear build <folder> -o <output>   # into another folder
nuclear check <folder> [--json]      # only the errors of every file
```

An error in any file stops writing: nothing is written.

`nuclear.toml` is optional when the dialect declares a file that marks its projects (`[project] marker`, §3):
the folder that has it is a project of that dialect, its name is the folder's (in snake_case) unless `app(name)`
says otherwise, and its output, `[project] output`. Every Dragonfly dialect marks its projects with `config.nuc`,
whose `dragonfly(...)` names the dialect (§3.1).

### 1.1 The project card: `.nuclear/project.json`

Every tool that builds a project (`nuclear build`, `nuclear check`, `dragonfly build`, the editors when a file of the
project is opened or saved) writes `.nuclear/project.json`, the card of the project: what it is, for an editor
client or a script that must not read Nuclear or load the dialect. It is derived from the manifest and the dialect
(never edited by hand; rewritten when it changes):

```json
{
  "name": "tasks_api",
  "platform": "backend",
  "dialect": "fastapi",
  "language": "python",
  "output": "build",
  "plugins": ["sqlite", "ollama"],
  "kinds": { "controller": "platform:backend", "data": "language:python", "jobs": "package:jobs 1.2.0" },
  "toolchain": { "url": "http://localhost:8000", "serve": null, "installed": ".venv" }
}
```

`toolchain` copies three keys of the dialect's `[toolchain]` (spec/dialects.md): where the running app answers, the
command an editor runs to serve it itself, and the folder whose presence means the dependencies are installed.

`platform` is `null` for a project of a language dialect. `kinds` lists every kind the project can use with its
source (ADR-0095): the language dialect, the platform's contract or a kind package. The editors read it for the
status bar; the language server itself reads the manifest (`config.nuc` or `nuclear.toml`) and checks each project
with its own dialect, whatever the editor's default dialect is.

Next to the card, the same commands write `.nuclear/map.md`, the **project map**: every declaration of the project
grouped by component, with its kind, its description (its doc comment) and its members, in Nuclear (aliases as
`list() = ListTasks.exec`, decorators as written). It is what an agent reads before it writes — the business of each
component without opening every file — and it is the same for every dialect of the platform.

### 1.2 The contract: `.nuclear/contract.json` and `contracts(...)`

The same call also writes `.nuclear/contract.json` (ADR-0098): what the project exposes or calls, in Nuclear's
notation, the same whatever the dialect. A backend lists its `controllers` (path, routes with verb, path, parameters
and where each goes: `path`, `query` or `body`, the answer and error types, the status) and a mobile or web app its
`repositories` (the calls it makes); both list the `models` those carry (fields with type, `optional`, `default`,
`json` name, `rules` such as `minLength`, the `key` field, doc comments as `description`). A backend also gets
`.nuclear/openapi.json` (OpenAPI 3.1 from the contract); both files are copied into the output folder and the
runtimes serve them at `GET /.nuclear/contract.json` and `GET /.nuclear/openapi.json`.

A client declares the backends it talks to in its manifest, with the name of the connection its calls use:

```nuclear
dragonfly(platform = "mobile", dialect = "flutter")
app(name = "tasks_app", version = "1.0.0+1")
contracts(tasks_api = "../tasks_api")              // a backend built next to this project (its .nuclear/contract.json)
contracts(tasks_api = "https://api.example.com")   // or the running API: nuclear contracts update fetches it
```

Every build writes `contracts/<name>/` from the contract: `models/<model>.nuc` for each model a route carries,
`models/<name>_error.nuc` (`<Name>Error`, the error every call answers with) and one `repository` per controller
with a method per route (`{id}` → `:id`; `@path`/`@query`/`@body` when a parameter is not where the verb would put
it; `Result<<Name>Error, T>`; the backend's doc comments). Only decorators the client's dialect consumes are written
(`@primary` and the validation rules where they exist; a GET that answers a keyed model gets `local = true` where
the local database exists). The files say they are generated and are rewritten on every build (a path contract) or
by `nuclear contracts update` (a url contract, which keeps a copy in `contracts/<name>/contract.json`: commit it, a
build never needs the network). A contract that cannot be read is `N0307`; a field the backend renamed is unknown
in the client at its next build (`N0116`).

```sh
nuclear contracts list            # each contract, its source and the version of the local copy
nuclear contracts update [name]   # fetch (url) or read (path) and regenerate contracts/<name>/
```

Example: `examples/examples-flutter-dialect/tasks_app` over `examples/examples-backend/tasks_api`.

## 2. Files and names

- Each `x.nuc` is written to `<sources>/x.<extension>` with the same relative path: with
  `[project] sources = "lib"` in the dialect, `components/users/data/user_repository.nuc` →
  `lib/components/users/data/user_repository.dart`.
- Hidden folders and the output folder are not read.
- A name is declared only once in the whole project (`N0303`).
- Every name declared in a file can be used in the others without `use`: the transpiler writes the import
  of the file that declares it. The dialect's `project_import` template gives the import path (Dart:
  `package:{{project}}/{{path}}.dart`).
- A file is imported only if the generated code uses one of its names (comments and the text of strings do
  not count; their interpolations do).
- What holds for the declarations of one file holds across files: `Result` propagation, aliases
  (`f() = Module.method` with `Module` in another file), overrides of bases in another file and named
  arguments.
- The `Result` runtime is written once, in `<sources>/nuclear/result.<extension>`, and the files that use
  it import it (if the dialect has its own runtime; Dart uses fpdart).
- `main` (the entry point) is never imported from another file.
- What the files of a `[project.outputs]` folder declare (the tests) is only visible to files of that same
  folder.

## 3. What the dialect adds: `[project]`

```toml
[project]
sources = "lib"
assets = ["assets"]                       # folders copied as they are to the output (text files)
marker = "main.nuc"                       # the folder with this file is a project without nuclear.toml
output = "build/app"                      # output of that project (default: build)

# the kind of declarations without a header word, by folder (* = any folder;
# the pattern also applies to subfolders)
[project.kinds]
"components/*/domain/models" = "model"

# folders that do not go to `sources`: test/x.nuc → <output>/test/x.dart
[project.outputs]
"test" = "test"

# files generated once per project from all the declarations
[project.files."lib/registry.g.dart"]
names = ["startApp"]                      # names it declares: the project's code can use them
template = { file = "templates/registry.tpl" }
```

The text files of each `assets` folder (JSON dictionaries, data) are written to the output with the same
path (`assets/i18n/en/shared.json`), next to the generated files.

### 3.1 Manifest in Nuclear: `[project.manifest]`

```toml
[project.manifest]
file = "config.nuc"                                          # at the project root
sections = ["app", "environment", "dependencies", "devDependencies"]
packages = ["dependencies", "devDependencies"]                # sections that list target packages
reserved = ["fpdart", "build_runner"]                        # packages the framework adds by itself
```

The file has only top-level calls to those sections, with named arguments whose value is a string, a
number or a boolean (`dependencies(hive = "^2.2.3")`). It is checked with the project (another section, an
unnamed argument or a value that is not a literal is `N0303`) and it is not transpiled: it has no output
(empty `out_path`). The framework's CLI reads it with `project::read_config` (section → arguments as text)
to write the target's manifest (the `pubspec.yaml` in Flutter).

A `packages` section only lists third-party packages the framework does not use: a package of the dialect
(its `defs`, not the ones the project added) or of `reserved` is `N0303` on its line
(`project::check_config`, `project::reserved_package`).

A backend dialect has the `dependencies` section too (`dependencies(nuclear_graph = ">=0.1")`): each package goes to
the generated manifest (`pyproject.toml`, `package.json`) as the template variable `dependencies` (`name`,
`version`, `path`); a version that starts with `.` or `/` is a folder on disk, resolved from the project root
(`nuclear_graph @ file:///...`, `"file:/..."`).

### 3.2 Type definitions of the project: `build/defs.lock` and `defs/`

The packages a project adds (besides the dialect's `defs`) are listed in `build/defs.lock`, one per line
(`hive@2.2.3`), or kept in the project itself under `defs/<language>/<name>/<version>/` (a package of the project's
own code, or one no cache has). The framework's CLI writes the lock: it resolves the versions and generates the
missing definitions into the cache (`~/.cache/nuclearlang/defs`, [definitions.md](definitions.md) §1). Every tool
that transpiles or checks the project (`nuclear build|check`, `dragonfly`, the LSP) adds both to the dialect with
`project::with_project_defs(root, dialect)` (`project_defs` lists them); a package that cannot be loaded is a
warning.

A kind by folder applies only to PascalCase declarations that are neither enums nor functions and carry no
other header word.

Variables of a `[project.files]` template:

| Variable | Content |
|---|---|
| `project` | the manifest's `name` |
| `decls` | each declaration: `name`, `kind`, `component`, `path`, `import`, `params` (constructor parameters: `name`, `type`...), `decor` |
| `kinds.<kind>` | the declarations of that kind |
| `decorated.<decorator>` | the declarations with that decorator |
| `files` | one `import` per file whose names the generated code writes |
| `has.<kind>`, `has.@<decorator>` | the project has at least one declaration of that kind or with that decorator |
| `components` | each component (`name`) with its declarations by kind (`kinds.<kind>`): a framework module per component |
| `uses.<Name>` | the generated code of the project writes that name (`{{#uses.Postgres}}` asks for its driver) |

Each declaration of `decls` (and of `kinds.<kind>`) also brings `generics` (its type parameters: `name`),
`fields` (`name`, `type`, `tinfo`, `optional`, `decor`) and `methods` (`name`, `owner`, `owner_decor`,
`decor`, `deco_args`, `params_list`, `param_groups`, `ret_info`).

### 3.3 Plugins: `[project.plugins]` and `plugins(...)` in `config.nuc`

A dialect lists the optional parts of its framework (Genkit, Kafka, LangChain, a database driver). A project carries
a plugin only when its `config.nuc` enables it, so it never gets heavier than what it uses (ADR-0092):

```toml
[project.manifest]
file = "config.nuc"
sections = ["plugins"]                 # Flutter: also app, environment, dependencies, devDependencies

[project.plugins.genkit]
description = "Genkit Dart in the app"
packages = { genkit = "^1.0.1", dragonfly_genkit = "local" }   # "local": a package of NuclearLang's packages/
dev_packages = {}
defs = ["genkit@1.0.1"]                # type definitions: their names need the plugin
kinds = ["flow", "chat"]               # kinds that need it
names = ["GoogleAI"]                   # more names that need it (classes of the runtime)
requires = []                          # plugins it enables too

[project.plugins.googleAI]
packages = { genkit_google_genai = "^1.0.1" }
requires = ["genkit"]
names = ["GoogleAI"]
```

```nuclear
// config.nuc
plugins(genkit = true, googleAI = true)
```

- `plugins(...)` takes only plugins of the dialect, with `true` or `false` (N0303); `requires` enables the others.
- A kind, or a name of a plugin's `defs` or `names`, used without enabling it is **N0304**, with the line to add.
  The plugin's type definitions are always loaded (their index only), so an editor still completes them.
- The templates of `[project.files]` get `plugins.<name>` (enabled), `plugins.packages` and `plugins.dev_packages`
  (`name`, `version`, `local`): a `pyproject.toml` or `package.json` lists only the enabled packages, and a file
  whose template renders nothing (`{{#plugins.genkit}}...{{/plugins.genkit}}`) is not written.
- A **kind package** of the platform (`kinds/<name>/`, [dialects.md](dialects.md) §7) is a plugin too: its `[plugin]`
  table gives the packages, defs and names; its kinds are the kinds it defines. One without an implementation for
  the project's dialect is N0306 when enabled.
- The framework's CLI adds the packages of the enabled plugins to the target's manifest and removes those of a
  plugin turned off (Flutter: `dragonfly build`, `build/app/.dragonfly_plugins`). `project::enabled_plugins(root,
  dialect)` gives the list.

Other variables of `[project.files]` templates: `components` (each component, `name`, with its declarations by kind:
`kinds.<kind>`) and `uses.<Name>` (the generated code writes that name).

## 4. Architecture rules: `[architecture]`

The dialect declares which declarations may use others; the transpiler checks it across the whole project
(it is the framework's linter):

```toml
[architecture]
scope = "components"                   # each subfolder of components/ is a component
public = ["appcomponent", "model", "@screen"]  # kinds (and @decorators) visible from other components
exempt = ["tests"]                     # kinds that may use any declaration

[architecture.callers]
usecase = ["appcomponent"]             # a usecase is only used from an appcomponent
service = ["usecase", "aggregate"]

[architecture.names]
BuildContext = ["stateless", "stateful"]  # a name that is not the project's, only in those kinds
```

| Rule | Code |
|---|---|
| A declaration uses one of a `callers` kind without being of an allowed kind | `N0301` |
| A declaration with a kind uses a `names` name without being of an allowed kind | `N0301` |
| A declaration uses something of another component whose kind is not in `public` | `N0302` |

## 5. Data for kind templates

A framework's kinds generate code from decorators and signatures. Besides the usual variables
(`nuclear spec`), the templates receive:

| Variable | Where | Content |
|---|---|---|
| `decor` | kind, `method`, `method_signature`, `function`, `field`, parameters | each decorator → `arg` (first positional argument), `args` (all), `items` (elements of a list argument), `path` (a first text argument as a route: its `{x}` segments written as `[options] route_param` says, `":name"` for NestJS; `@get("{id}")` → `decor.get.path` = `:id`) and each named argument. A named argument that is a list also gives its `items` (`decor.screen.providers.items`), and one that is an object, each field (`decor.screen.loggedIn.forward`); `{{decor.screen.providers}}` is still its code |
| `deco_args` | kind, methods, functions | the named arguments of all its decorators together |
| `owner_decor` | methods, `constructor`, `field` | `decor` of the class |
| `owner` | methods, `field` | name of the class |
| `params_list` | kind, methods, functions | each parameter: `name`, `type`, `tinfo`, `optional`, `named`, `decor`, `decorated` |
| `param_groups` | methods | `plain` (no decorator) and one group per decorator (`path`, `query`), each `{ list }` |
| `call_args` | methods | the parameters as the arguments of a call (`id, name: name`) |
| `ret_info`, `tinfo` | methods, functions, parameters | of the type: `type`, `list`, `item`, `name` (name in Nuclear, without type arguments), `args` (the type arguments, each with this same data: `Page<Character>`), `generic` (a type parameter of the declaration: `T`), `model` (a class of the program, or of the type definitions that reads itself from JSON: the runtime's `PageOf`), `runtime` (a `model` of the type definitions, not of the program), `fields_decor.<decorator>` and `decor` of a model (its first field with that decorator: `primary` → `id`; its own decorators: `decor.entity.arg`, the table), `primitive`, `void`, `nullable`, `is.<name>`; of a `Result<T, E>`: `result`, `full`, `err`, `err_is.<name>` and the rest describes `T` |
| `override` | `field` | the field replaces a member of a base |
| `file`, `project`, `component`, `kind` | kind | name of the file, the project, the component and the kind |
| `app.name`, `app.title`, `app.description`, `app.version` | project files | `app(...)` of `config.nuc` (`title`: the name when absent) |
| `asset.<path>` | project files | a file of the `[project] assets` folders exists, its path with `/`, `.` and `-` as `_` (`asset.public_styles_css`: index.html links the stylesheet only when the project has one) |

Kind fields for frameworks:

| Field | What it does |
|---|---|
| `async_methods = true` | all its methods are asynchronous (a `~` calls them) |
| `methods = ["exec"]` | the only allowed methods (`N0215`) |
| `returns = "Result"` | the type each method returns (`N0216`) |
| `file_header = "part '{{file}}.g.dart';"` | written once after the file's imports |
| `alias_via = "inject"` | an alias of the class takes the instance from that function: `inject<Module>().method(...)` |
| `declares = ["{{name}}Builder"]` | other classes the template writes: the program may use them |
| `provide = { template = "inject<{{name}}>()", kinds = ["usecase"] }` | inside the kind, a program declaration of those kinds is read with the template: `GetUser.exec()` → `inject<GetUser>().exec()` |
| `results = "keep"` | inside the kind, a `Result` never propagates: it is kept as a value (tests) |
| `void_methods = true` | its methods do not return their last line (tests) |
| `imports` | imported if the generated code writes that name |

`[decorators]` of the dialect:

| Key | What it does |
|---|---|
| `consumed` | decorators the templates read and that are not written to the target |
| `[decorators.provide]` | decorator (or `decorator.argument`) → template: with `"screen.providers" = "context.read<{{name}}>()"`, in a declaration with `@screen(providers = [Manager])`, `Manager.x()` is `context.read<Manager>().x()` |
| `[decorators.args]` | decorator → its named arguments: `["route", "loggedIn", "loggedIn.forward!"]` (`a.b` = field of object `a`; `!` = required). One that is not listed, or a required one that is missing, is `N0217` |
| `void` | a method with that decorator returns nothing to its caller |
