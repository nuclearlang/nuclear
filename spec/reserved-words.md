# Reserved words per language

Every language has a dictionary of reserved words:

- **Base language:** [languages/nuclear.toml](../languages/nuclear.toml). A test checks that it matches the
  lexer.
- **Target languages:** the `[keywords]` section of the target dialect, in
  [dialects/typescript/keywords.toml](../dialects/typescript/keywords.toml) and
  [dialects/dart/keywords.toml](../dialects/dart/keywords.toml). The dialects that extend them
  (nodejs, nestjs, flutter) inherit them.

A target's dictionary is used for:

1. **Coverage:** every reserved word of the target says how Nuclear reaches it. If a feature of the target
   has no path, a complete project cannot be exported.
2. **Collisions:** if a Nuclear name is a reserved word in the target (`delete = 1` in TypeScript), the
   emitter renames it according to `options.reserved_escape` (`delete_`).
3. **Tools:** the highlighter, the LSP and the MCP read these files to know which words exist in each
   dialect.

## Format

```toml
[keywords]
extension = { level = "builtin", via = "kind", nuclear = "extension NumberParsing on String" }
```

### `level`: how reserved the word is in that language

| Value | Meaning | Is an equal Nuclear name renamed? |
|---|---|---|
| `reserved` | Can never be used as an identifier | Yes |
| `strict` | Reserved in strict mode / modules (TS: `let`, `static`...) | Yes (Nuclear emits modules) |
| `builtin` | Dart: cannot be a type name, can be a variable | No |
| `contextual` | A keyword only in some places | No |

### `via`: how Nuclear produces it

| Value | Meaning | Example |
|---|---|---|
| `base` | Covered by the syntax of the base language | `if`, `for`, `extends` |
| `auto` | The transpiler generates it; the developer never writes it | `return`, `new`, `async`, `const`/`final` |
| `kind` | Header word that defines the kind of declaration (`[kinds.*]`) | `extension`, `mixin` |
| `modifier` | Header word before the name that modifies the declaration (`[modifiers]`) | `static`, `abstract` |
| `clause` | Word after the name, followed by a type (`[clauses]`), or after a `use` path | `with Flyer`, `show foo` |
| `type` | Name used in type position | `dynamic`, `never`, `unknown` |
| `builtin` | Function-like form the dialect provides | `assert(x > 0)`, `delete(obj.key)` |
| `none` | Not needed: another construct covers the case | `do` (covered by `while`) |
| `todo` | Still to design. **Blocks full coverage** | `namespace`, `operator` |

`nuclear` shows how it is written in Nuclear. `note` explains when needed.

## Header words

`kind`, `modifier` and `clause` words **are not reserved in Nuclear**. They only mean something in the
header of a declaration, and the parser accepts them without knowing them ("header + block" rule, see
nuclear-base.md §4). The active dialect validates them (`N0201`, `N0202`):

```nuclear
abstract Shape                     // modifier
extension NumberParsing on String  // kind + name + clause
```

Outside the header they are normal identifiers: `extension = "png"` is a valid variable (and if it
collides in the target, it is renamed).

A framework dialect (flutter, nestjs) has no dictionary of its own: it uses its target's and adds its
header words (`stateless`, `controller`...) in `[kinds.*]` (see [dialects.md](dialects.md)).
