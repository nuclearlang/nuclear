# tools/defs

Generates the type definitions packages of the `dart` language (`defs/dart/<package>/<version>/`), used by the
`dart` and `flutter` dialects. Format and rules: [spec/definitions.md](../../spec/definitions.md).

It uses the official Dart analyzer (`package:analyzer`), so the API is exact for the installed versions: the
Dart/Flutter SDK's and those of the packages of `flutter_project/pubspec.yaml`.

## Regenerate everything

```sh
FLUTTER_ROOT=~/develop/flutter tools/defs/generate.sh
```

It writes, with the installed versions:

| Package | Libraries |
|---|---|
| `dart@<Dart version>` | `dart:core` (implicit), `dart:async`, `dart:math`, `dart:convert`, `dart:collection`, `dart:io`, `dart:typed_data`, `dart:isolate`, `dart:developer` |
| `flutter@<Flutter version>` | `package:flutter/{widgets,material,cupertino,services,foundation,painting,rendering,gestures,animation,scheduler,semantics,physics}.dart` (in that order: each name goes to the most basic library) |
| `dio`, `get_it`, `json_annotation`, `bloc`, `flutter_bloc`, `go_router`, `connectivity_plus`, `flutter_secure_storage` | the main library of each package, with the version of `flutter_project/pubspec.lock` |
| `dragonfly` | `package:dragonfly/dragonfly.dart` and `testing.dart` (the runtime of `packages/dragonfly`, a path dependency) |

Then:

1. if a version changed, update `defs` in `dialects/dart/dialect.toml` or `dialects/mobile/flutter/dialect.toml`;
2. `cargo test` (validates the dialects; the golden files show whether an import changed);
3. `FLUTTER=$FLUTTER_ROOT/bin/flutter tools/analyze-flutter.sh` (transpiles the examples and checks them with
   `flutter analyze`).

## A new pub.dev package

1. Add it to `flutter_project`: `(cd tools/defs/flutter_project && $FLUTTER_ROOT/bin/flutter pub add go_router)`.
2. Add its line at the end of `generate.sh` (with `--depends`/`--exclude` of the packages it depends on, like
   `flutter_bloc`).
3. Generate it and name it in the `defs` of the dialect that uses it.

## A project's packages

`dragonfly flutter add <package>` uses this same generator (installed by `install.sh` in
`~/.local/share/nuclearlang/tools/defs`) on the project's app: `--project build/app`, the version of its
`pubspec.lock` and `--exclude` of the dialect's packages. It writes into the cache
`~/.cache/nuclearlang/defs/dart/` ([spec/definitions.md](../../spec/definitions.md) §6). There is no need to touch
`generate.sh` nor the dialect.

## `bin/defs.dart`

```sh
dart run bin/defs.dart --project <dir> --out <package folder> --name <n> --version <v>
    [--description <text>] [--depends <n@v>]... [--implicit <library>]... [--exclude <folder>]...
    <library>...
```

- `--project`: the folder whose `package_config` resolves the libraries (`.` for Dart's, `flutter_project` for
  `package:flutter/...` and the pub packages).
- `--exclude`: packages already generated (the dependencies): their declarations are not repeated and their
  libraries count for `reexports`.
- `--implicit`: libraries that are always visible (`dart:core`).

`flutter_project/` is an empty project that only resolves the packages; it is also where
`tools/analyze-flutter.sh` writes the generated code (`lib/generated/`).

## Other languages

The same format for every language; only the generator changes:

| Language | Generator | Regenerate |
|---|---|---|
| Python | `tools/defs/python/gen.py` (`inspect` over the installed modules of `tools/python/.venv`) | `tools/defs/python/generate.sh` |
| TypeScript | `tools/defs/typescript/gen.js` (the TypeScript compiler over the `.d.ts` in `tools/node/node_modules`; `--lib es2022` for the standard library) | `tools/defs/typescript/generate.sh` |

Both write `params` (the unnamed constructor) and `results` (methods that return a `Result`) in `index.json`.
