---
layer: L1
component: system
read-when: you need to know WHY something is the way it is, or you are about to take a new decision
---

# Architecture Decision Records

One file per decision (Michael Nygard's format). ADRs record *why*; the rules themselves live in the specs
and the `CONTEXT.md` files. They are not on the default reading path: open one only when its title answers
your question.

- A new decision gets the next number, starts from [`_template.md`](_template.md) and is added to this table.
- A decision is never rewritten: a new ADR supersedes it and both say so in `status`.
- `scope` is the repository the decision belongs to after the split (ADR-0083): `nuclear`, `dragonfly`,
  `dragonfly-flutter`, `editors` (the editor extensions).

| ADR | Title | Scope | Status |
|---|---|---|---|
| [0001](0001-file-extension.md) | File extension | `nuclear` | Accepted |
| [0002](0002-functions-without-keyword.md) | Functions without a keyword | `nuclear` | Accepted |
| [0003](0003-variables.md) | Variables | `nuclear` | Accepted |
| [0004](0004-logical-operators.md) | Logical operators | `nuclear` | Accepted |
| [0005](0005-implicit-return.md) | Implicit return | `nuclear` | Accepted |
| [0006](0006-implicit-self.md) | Implicit `this` | `nuclear` | Accepted |
| [0007](0007-no-visibility.md) | No visibility modifiers | `nuclear` | Accepted |
| [0008](0008-construct-with-pascalcase.md) | Construct with PascalCase | `nuclear` | Accepted |
| [0009](0009-quotes.md) | Quotes | `nuclear` | Accepted |
| [0010](0010-interpolation.md) | Interpolation | `nuclear` | Accepted |
| [0011](0011-indentation.md) | Indentation | `nuclear` | Accepted |
| [0012](0012-automatic-async.md) | Automatic async | `nuclear` | Accepted |
| [0013](0013-await-with-tilde.md) | Await with `~` | `nuclear` | Accepted |
| [0014](0014-imports-by-transpiler.md) | Imports written by the transpiler | `nuclear` | Accepted |
| [0015](0015-versioned-technologies.md) | Versioned technologies and optional agents | `nuclear` | Accepted |
| [0016](0016-not-object-oriented.md) | Not object oriented | `nuclear` | Accepted |
| [0017](0017-generated-code-context.md) | Generated context maps of the code | `nuclear` | Accepted, not implemented |
| [0018](0018-map-type.md) | Map type | `nuclear` | Accepted |
| [0019](0019-no-nested-functions.md) | No nested functions | `nuclear` | Accepted |
| [0020](0020-braced-parameters.md) | Parameters between `{ }` | `nuclear` | Accepted |
| [0021](0021-one-line-lambdas.md) | One-line lambdas | `nuclear` | Accepted |
| [0022](0022-block-scope.md) | Block scope and values | `nuclear` | Accepted |
| [0023](0023-explicit-parameters.md) | Only explicit parameters | `nuclear` | Accepted |
| [0024](0024-no-type-keyword.md) | No `type` | `nuclear` | Accepted |
| [0025](0025-dialect-specs.md) | Dialect specs | `nuclear` | Accepted |
| [0026](0026-enum-item-without-value.md) | Enum item without a value | `nuclear` | Accepted |
| [0027](0027-cast-non-null-generic-call.md) | `x as T`, `x!`, `X<T>(...)` | `nuclear` | Accepted |
| [0028](0028-bounded-generics.md) | Bounded generics | `nuclear` | Accepted |
| [0029](0029-generators.md) | Generators | `nuclear` | Accepted |
| [0030](0030-cascades.md) | Cascades | `nuclear` | Accepted |
| [0031](0031-no-records.md) | No records | `nuclear` | Accepted |
| [0032](0032-destructuring.md) | Destructuring | `nuclear` | Accepted |
| [0033](0033-if-in-lists.md) | `if` inside lists | `nuclear` | Accepted |
| [0034](0034-inheriting-constructors.md) | Constructor of an inheriting module | `nuclear` | Accepted |
| [0035](0035-external-classes.md) | External classes | `nuclear` | Accepted |
| [0036](0036-results.md) | Results | `nuclear` | Accepted; order amended by ADR-0047 |
| [0037](0037-function-aliases.md) | Function aliases | `nuclear` | Accepted |
| [0038](0038-golden-rule.md) | Golden rule | `nuclear` | Accepted |
| [0039](0039-flutter-dialect-as-framework.md) | The `flutter` dialect as a framework | `dragonfly-flutter` | Accepted |
| [0040](0040-type-definitions.md) | Cross-dialect type definitions | `nuclear` | Accepted |
| [0041](0041-dragonfly.md) | Dragonfly | `dragonfly` | Accepted; generalized by ADR-0072 |
| [0042](0042-component-structure.md) | Component structure | `dragonfly` | Accepted |
| [0043](0043-dependency-injection.md) | Dependency injection | `dragonfly` | Accepted |
| [0044](0044-json.md) | JSON | `dragonfly-flutter` | Accepted |
| [0045](0045-separate-project.md) | Separate project | `dragonfly` | Accepted |
| [0046](0046-architecture-rules.md) | Architecture rules (linter) | `dragonfly` | Accepted |
| [0047](0047-result-order-dart-runtime.md) | `Result` order and Dart runtime | `nuclear` | Accepted |
| [0048](0048-ok-error-by-callback-return.md) | `.ok()` / `.error()` by what the callback returns | `nuclear` | Accepted |
| [0049](0049-tests-in-nuclear.md) | Tests in Nuclear | `dragonfly-flutter` | Accepted |
| [0050](0050-nuc-only-project.md) | A Dragonfly project is only `.nuc` | `dragonfly` | Accepted |
| [0051](0051-dragonfly-package-and-dialect.md) | Dragonfly package and `flutter` dialect | `dragonfly-flutter` | Accepted |
| [0052](0052-global-events.md) | Global events | `dragonfly` | Accepted |
| [0053](0053-global-components.md) | Global components | `dragonfly` | Accepted |
| [0054](0054-session.md) | Session | `dragonfly` | Accepted |
| [0055](0055-screen-decorator.md) | `@screen` | `dragonfly-flutter` | Accepted |
| [0056](0056-acl.md) | ACL | `dragonfly` | Accepted |
| [0057](0057-routes.md) | Routes | `dragonfly-flutter` | Accepted |
| [0058](0058-forms.md) | Forms | `dragonfly` | Accepted |
| [0059](0059-dialogs.md) | Dialogs | `dragonfly-flutter` | Accepted |
| [0060](0060-translations.md) | Translations | `dragonfly` | Accepted |
| [0061](0061-splash.md) | Splash | `dragonfly-flutter` | Accepted |
| [0062](0062-generic-models.md) | Generic models | `dragonfly` | Accepted |
| [0063](0063-native-splash.md) | Native splash | `dragonfly-flutter` | Accepted |
| [0064](0064-json-names-and-converters.md) | JSON names and converters | `dragonfly-flutter` | Accepted |
| [0065](0065-manifest-in-nuclear.md) | Manifest in Nuclear (`config.nuc`) | `dragonfly-flutter` | Accepted |
| [0066](0066-third-party-packages-defs.md) | Third-party packages with definitions | `dragonfly-flutter` | Accepted |
| [0067](0067-commands-per-dialect.md) | Commands per dialect | `dragonfly` | Accepted |
| [0068](0068-debug-nuc.md) | Debugging `.nuc` | `editors` | Accepted |
| [0069](0069-only-third-party-packages.md) | Only third-party packages | `dragonfly-flutter` | Accepted |
| [0070](0070-no-empty-config-sections.md) | No empty sections in `config.nuc` | `dragonfly-flutter` | Accepted |
| [0071](0071-assets-by-convention.md) | Assets by convention | `dragonfly-flutter` | Accepted |
| [0072](0072-dragonfly-language-independent.md) | Dragonfly is language independent | `dragonfly` | Accepted |
| [0073](0073-dialects-with-agents.md) | New dialects with agents | `nuclear` | Accepted |
| [0074](0074-dragonfly-device.md) | Dragonfly device | `editors` | Accepted |
| [0075](0075-code-in-english.md) | Code in English, documentation in Spanish | `nuclear` | Superseded in part by ADR-0082 |
| [0076](0076-mixed-debugging.md) | Mixed debugging | `editors` | Accepted |
| [0077](0077-editor-extensions-per-framework.md) | Editor extensions per framework | `editors` | Accepted |
| [0078](0078-component-logs.md) | Readable logs by component | `dragonfly-flutter` | Accepted |
| [0079](0079-editor-documentation.md) | Editor documentation and doc comments | `nuclear` | Accepted |
| [0080](0080-comment-lines-with-star.md) | Comment lines with ` * ` | `nuclear` | Accepted |
| [0081](0081-cli-devices.md) | Dragonfly devices from the CLI | `dragonfly` | Accepted |
| [0082](0082-documentation-in-english.md) | Documentation in English | `nuclear` | Accepted |
| [0083](0083-repository-split.md) | Repository split | `nuclear` | Accepted |
| [0084](0084-layered-ai-context.md) | Layered AI context | `nuclear` | Accepted |
| [0085](0085-dragonfly-proposals-accepted.md) | Dragonfly proposals accepted | `dragonfly-flutter` | Accepted |
| [0086](0086-operators-and-the-targets-library.md) | Operators, literals and the target's library | `nuclear` | Accepted |
| [0087](0087-agent-support.md) | Agent support: brief, skill, agent init, hook, evaluation | `nuclear` | Accepted |
| [0088](0088-compiler-corrects-agents.md) | The compiler corrects agents: habits, invented APIs, words on demand, llms.txt | `nuclear` | Accepted |
| [0089](0089-python-dialect.md) | Python dialect and the generic emitter options it needed | `nuclear` | Accepted |
| [0090](0090-dragonfly-python-langgraph.md) | Dragonfly for Python: the langgraph dialect (FastAPI, SQLAlchemy, pydantic, LangChain, LangGraph) | `dragonfly` | Accepted |
| [0091](0091-dragonfly-nestjs-sequelize.md) | Dragonfly for NestJS: the nestjs dialect with Sequelize, Kafka, Redis and LangChain.js | `dragonfly` | Accepted |
| [0092](0092-plugins-and-genkit.md) | Plugins in config.nuc, and Genkit for Dragonfly Flutter | `dragonfly` | Accepted |
| [0093](0093-platforms-and-dialects.md) | Three platforms (web, mobile, backend); dialects grouped under them, never visible in the Nuclear code | `dragonfly` | Accepted |
| [0094](0094-standard-library.md) | Nuclear has its own standard library, translated by every dialect | `nuclear` | Accepted |
| [0095](0095-kinds-as-libraries.md) | Kinds are Nuclear's libraries: native kinds of the platform, extension kinds as packages | `dragonfly` | Accepted |
| [0096](0096-mobile-local-database.md) | Local database of the mobile platform: `@primary` models, `@repository(saveLocal)` and `<method>AsLocal()` | `dragonfly-mobile` | Accepted |
| [0097](0097-web-platform-react.md) | The web platform: the mobile platform in a browser, first dialect `react` | `dragonfly-web` | Accepted |
| [0098](0098-shared-contracts.md) | Shared contracts: a backend publishes what it exposes, a client generates its models and repositories | `dragonfly` | Accepted |
| [0099](0099-editor-workspace.md) | The Nuclear workspace in the editor: projects as tabs, tools full screen, keyboard first | `editors` | Accepted |
| [0100](0100-backend-data.md) | Backend data: one key decorator, TypeORM relations, pagination, transactions, migrations | `dragonfly` | Accepted |
| [0101](0101-toolchain-data-and-installers.md) | Run, debug and install from data: `[toolchain]`, the card, the release installers | `dragonfly` | Accepted |
| [0102](0102-ai-loop.md) | The AI loop: Nuclear Smart Graph (private RAG + MCP), Nuclear Developer (local 3B agents), the Loop tool | `nuclear` | Accepted |
| [0103](0103-generators.md) | The CLI and the MCP generate the code (`dragonfly gen`); the agent writes only what is not standard; HTTP repositories in the backend | `dragonfly` | Accepted |
