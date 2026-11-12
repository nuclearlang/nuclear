# Type definitions

**Type definitions** describe the public API of the libraries of a target language: classes,
constructors, parameters, types, members, enums, extensions, functions and documentation. They are the
equivalent of TypeScript's `.d.ts` files and `@types/*` packages: one package per library and version,
stored in a repository shared by every dialect of the same language.

They are used by:

- the **transpiler**: automatic imports, overrides (`override`), members inherited from external classes,
  `implements` of external interfaces and error `N0106`;
- the **tools**: the Language Server (completion, hover, parameter signatures), `nuclear defs` and agents,
  which look up an API without reading the library's source code.

The system is the same for every language. The only language-specific part is the generator that
produces the files (§6).

## 1. Repository

A package is a folder `<root>/<language>/<package>/<version>/` with three files:

```
defs/
└── dart/                          # language: the target dialect of the chain (dart, typescript...)
    ├── dart/3.13.5/               # Dart SDK
    ├── flutter/3.47.7/            # Flutter SDK
    ├── flutter_bloc/9.1.1/
    ├── bloc/9.2.1/
    ├── dio/5.11.1/
    ├── get_it/9.3.0/
    └── json_annotation/4.12.0/
        ├── package.toml           # manifest
        ├── index.json             # what the transpiler reads on every build
        └── api.json               # the full API, for the tools
```

The language of a dialect is the target dialect of its chain: `flutter` extends `dart`, so it uses the
packages of `defs/dart/`.

Roots are searched in this order, and the first one that has the package wins:

1. `--defs-path <dir>` (repeatable) and the `NUCLEAR_DEFS_PATH` variable (separated by `:`);
2. the `defs/` folder next to each dialects folder: `./defs` (next to `./dialects`), the one in the
   NuclearLang repository and the one shipped with the Language Server;
3. the cache of packages generated for projects: `$XDG_CACHE_HOME/nuclearlang/defs` or
   `~/.cache/nuclearlang/defs` (`nuclear_dialect::defs_cache_dir`), shared by every project.

A project keeps its own packages (or a different version of an included one) in
`<project>/defs/<language>/<name>/<version>/`; `nuclear build|check` and the LSP load every package found there.
Packages a project adds are loaded on top of the dialect's: the project lists them in
`build/defs.lock` ([projects.md](projects.md) §3.2) and the tools add them with `Loader::add_defs`, which
marks them `from_project` (so they do not count as packages of the dialect).

## 2. Using them from a dialect

```toml
[dialect]
name = "flutter"
extends = "dart"
defs = ["flutter@3.47.7", "flutter_bloc@9.1.1", "get_it@9.3.0", "dio@5.11.1", "json_annotation@4.12.0"]
```

- `defs` is a list of `name@version`. A dialect adds its packages to its parent's.
- The dependencies of a package (`depends` in its `package.toml`) are loaded automatically, after it:
  `flutter_bloc@9.1.1` brings `bloc@9.2.1`.
- Each package is loaded with a single version. Two versions of the same package in the chain is error
  `N0214`; a package missing from the repository is error `N0213`, with the available versions.

**Automatic imports.** Every public name of a package's libraries is imported when the code uses it. If
two packages export the same name, the first one loaded keeps it:

1. the parent dialect's packages before the child's;
2. a dialect's packages in the order of `defs`, each before its dependencies;
3. inside a package, the first library of `libraries` that exports the name (the most basic one).

`implicit` libraries (always visible in the target, like `dart:core`) are never imported, and their names
cannot be imported from another package. A name of `[auto_imports]` or `[functions]` written in a dialect
file has priority over the packages. `reexports` avoids importing a library that another imported library
already includes.

Example, with the `flutter` dialect: `Uint8List` → `dart:typed_data` (dart is loaded first),
`Scaffold` → `package:flutter/material.dart`, `Cubit` → `package:flutter_bloc/flutter_bloc.dart`
(flutter_bloc is loaded before bloc and re-exports it), `Dio` → `package:dio/dio.dart`.

## 3. Format

The files are written by a generator (§6); they are not edited by hand. `index.json` and `api.json` carry
`"format": 1`. Types are written in NuclearLang notation, whatever the target language:

| Target (Dart) | Definitions |
|---|---|
| `String`, `bool`, `num`, `int`, `double` | `string`, `boolean`, `number`, `int`, `double` |
| `List<Widget>`, `Map<String, dynamic>` | `Widget[]`, `<string, dynamic>` |
| `TextStyle?` | `TextStyle?` |
| `void Function(int value)` | `(value: int) => void` |
| `Future<Response<T>>`, `VoidCallback` | `Future<Response<T>>`, `VoidCallback` (aliases are kept) |

### 3.1 `package.toml`

```toml
[package]
name = "flutter"
version = "3.47.7"
language = "dart"
description = "Flutter SDK 3.47.7: widgets, material, cupertino..."
depends = ["dart@3.13.5"]
libraries = ["package:flutter/widgets.dart", "package:flutter/material.dart", ...]   # priority order
implicit = []                                                                         # dart: ["dart:core"]
```

### 3.2 `index.json`

The minimum the transpiler needs, so that loading it on every build is fast:

```json
{
"format": 1,
"libraries": ["package:flutter/widgets.dart", "package:flutter/material.dart"],
"reexports": {"package:flutter/material.dart": ["package:flutter/widgets.dart"]},
"symbols": {
  "Scaffold": 1,
  "Text": 0
},
"classes": {
  "State": {"supers": ["Object", "Diagnosticable"], "members": ["build", "context", "initState", "setState"]},
  "Comparable": {"kind": "interface", "supers": ["Object"], "members": ["compareTo"]}
}
}
```

- `symbols`: each public name → index of the library that provides it.
- `classes`: each class (also the private bases inheritance goes through) with its direct supertypes and
  the names of its own instance members. `kind` appears only on classes that cannot be extended:
  `"interface"` (goes to `implements`) or `"mixin"` (goes to `with`).

Each entry of `classes` may also have `params` (the parameter names of the unnamed constructor; a named argument that
is not one is N0219) and `results` (method → its `Result<E, T>` return type, so the transpiler unwraps
`x = ~repo.find(id)` without reading `api.json`).

### 3.3 `api.json`

One declaration per line:

```json
{"format": 1, "declarations": [
{"name":"Text","library":"package:flutter/widgets.dart","kind":"class","extends":"StatelessWidget","doc":"A run of text with a single style.","constructors":[{"name":"","const":true,"params":[{"name":"data","type":"string"},{"name":"style","type":"TextStyle?","named":true,"optional":true}]},{"name":"rich","const":true,"params":[...]}],"members":[{"name":"data","kind":"property","type":"string?","readonly":true},{"name":"build","kind":"method","params":[{"name":"context","type":"BuildContext"}],"returns":"Widget"}]}
]}
```

| Field | Content |
|---|---|
| `name`, `library` | Name and the library that provides it (no `library`: a private base) |
| `kind` | `class`, `mixin`, `enum`, `extension_type`, `extension`, `function`, `variable`, `typedef` |
| `modifiers` | Class modifiers of the target: `abstract`, `sealed`, `interface`, `base`, `final`, `mixin` |
| `type_params` | `["T"]`, `["T: Comparable<T>"]` |
| `extends`, `with`, `implements`, `on` | Supertypes; `on` is a mixin's constraint or an extension's type |
| `constructors` | `name` (`""` = unnamed, `"rich"` = `Text.rich`), `const`, `factory`, `params` |
| `members` | `name`, `kind` (`property`, `method`, `operator`), `static`, `type` (property), `readonly`, `type_params`, `params`, `returns` |
| `params` | `name`, `type`, `named` (goes between `{ }`), `optional`, `default` (written in the target) |
| `values` | Values of an enum |
| `type` | Type of a variable or the aliased type (`typedef`) |
| `returns`, `params` | Signature of a function; `returns` types the value of a call (`env("PORT")` is a `string`, so `env("PORT").toInt()` is the library's `toInt`) |
| `doc`, `deprecated` | First paragraph of the documentation; the declaration is deprecated |

## 4. What the transpiler does with them

It reads only each package's `index.json`:

- automatic imports and `reexports` (§2);
- when a module extends or implements a class that is not in the file, it walks its `supers`: a method
  named like an inherited member overrides it (`override` / `override_interface`) and inherited members are
  used without declaring them;
- a name that is not in the file, the class chain or the imports is error `N0106`; a base that is not in
  the definitions leaves the module open (no `N0106` nor `override`);
- a base with `kind = "interface"` goes to `implements`;
- `[kinds.<k>] inherits = ["State"]` takes from the definitions the members and overrides of the class
  the kind generates.

`api.json` is read only when a tool asks for the full API, once per process.

## 5. Tools

```sh
nuclear defs list                              # packages of the repository
nuclear defs list --dialect flutter            # packages a dialect uses, in priority order
nuclear defs show Text --dialect flutter       # declaration in NuclearLang notation
nuclear defs show Text --dialect flutter --inherited   # + inherited and extension members
nuclear defs show Dio.get --dialect flutter    # a member or a named constructor (Text.rich)
nuclear defs show Dio --dialect flutter --json # the declaration as it is in api.json
nuclear defs search snackbar --dialect flutter # declarations whose name contains the text
```

```
$ nuclear defs show Dio.get --dialect flutter
// Dio.get
// Convenience method to make an HTTP GET request.
get<T>(
  path: string,
  {
    data?: Object,
    queryParameters?: <string, dynamic>,
    options?: Options,
    cancelToken?: CancelToken,
    onReceiveProgress?: ProgressCallback,
  },
) Future<Response<T>>
```

Parameters are shown as in a NuclearLang signature: the ones between `{ }` are passed by name
(`dio.get(url, options = opts)`), `name?: T` is optional and `name: T = value` has a default.

The Language Server and any tool written in Rust use the same API: `Dialect.typedefs` (`get(name)`,
`members(class)` with the inherited ones, `declarations()`, `extensions_on(type)`) and the functions
`render`, `render_member`, `render_constructor` and `render_param` of `nuclear_dialect::defs`.

## 6. Generating packages

Each language has a generator that writes the three files from the official source of the API:

| Language | Generator | Source |
|---|---|---|
| `dart` | `tools/defs` (`bin/defs.dart`, `generate.sh`) | the official Dart analyzer (`package:analyzer`) over the installed SDK and the packages of `tools/defs/flutter_project/pubspec.yaml` |

A new version of a library is a new folder; the dialect changes its reference in `defs`. Previous versions
are kept for the projects that use them.

For a dialect to include a pub.dev package (`go_router`...): add it to
`tools/defs/flutter_project/pubspec.yaml`, add its line in `tools/defs/generate.sh`, generate it and name it
in the dialect's `defs`.

For a package of a single project, the framework's CLI generates it: `dragonfly flutter add hive` runs
`bin/defs.dart` over the app (`--project build/app`) with the resolved version, excludes the packages the
dialect already loads (`--exclude`) and describes `package:<n>/<n>.dart` (or each file of `lib/` if it does
not exist). The result goes to the cache (§1) and is reused by every project; removing the package does not
delete it. `install.sh` installs the generator in `<install>/tools/defs`.
