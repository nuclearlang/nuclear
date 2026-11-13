# Decisions

Every decision of the platform is an Architecture Decision Record in [`docs/adr/`](../docs/adr/README.md),
one file per decision with its status and the repository it belongs to. This page is the index.

| ADR | Title | Scope | Status |
|---|---|---|---|
| [0001](../docs/adr/0001-file-extension.md) | File extension | `nuclear` | Accepted |
| [0002](../docs/adr/0002-functions-without-keyword.md) | Functions without a keyword | `nuclear` | Accepted |
| [0003](../docs/adr/0003-variables.md) | Variables | `nuclear` | Accepted |
| [0004](../docs/adr/0004-logical-operators.md) | Logical operators | `nuclear` | Accepted |
| [0005](../docs/adr/0005-implicit-return.md) | Implicit return | `nuclear` | Accepted |
| [0006](../docs/adr/0006-implicit-self.md) | Implicit `this` | `nuclear` | Accepted |
| [0007](../docs/adr/0007-no-visibility.md) | No visibility modifiers | `nuclear` | Accepted |
| [0008](../docs/adr/0008-construct-with-pascalcase.md) | Construct with PascalCase | `nuclear` | Accepted |
| [0009](../docs/adr/0009-quotes.md) | Quotes | `nuclear` | Accepted |
| [0010](../docs/adr/0010-interpolation.md) | Interpolation | `nuclear` | Accepted |
| [0011](../docs/adr/0011-indentation.md) | Indentation | `nuclear` | Accepted |
| [0012](../docs/adr/0012-automatic-async.md) | Automatic async | `nuclear` | Accepted |
| [0013](../docs/adr/0013-await-with-tilde.md) | Await with `~` | `nuclear` | Accepted |
| [0014](../docs/adr/0014-imports-by-transpiler.md) | Imports written by the transpiler | `nuclear` | Accepted |
| [0015](../docs/adr/0015-versioned-technologies.md) | Versioned technologies and optional agents | `nuclear` | Accepted |
| [0016](../docs/adr/0016-not-object-oriented.md) | Not object oriented | `nuclear` | Accepted |
| [0017](../docs/adr/0017-generated-code-context.md) | Generated context maps of the code | `nuclear` | Accepted, not implemented |
| [0018](../docs/adr/0018-map-type.md) | Map type | `nuclear` | Accepted |
| [0019](../docs/adr/0019-no-nested-functions.md) | No nested functions | `nuclear` | Accepted |
| [0020](../docs/adr/0020-braced-parameters.md) | Parameters between `{ }` | `nuclear` | Accepted |
| [0021](../docs/adr/0021-one-line-lambdas.md) | One-line lambdas | `nuclear` | Accepted |
| [0022](../docs/adr/0022-block-scope.md) | Block scope and values | `nuclear` | Accepted |
| [0023](../docs/adr/0023-explicit-parameters.md) | Only explicit parameters | `nuclear` | Accepted |
| [0024](../docs/adr/0024-no-type-keyword.md) | No `type` | `nuclear` | Accepted |
| [0025](../docs/adr/0025-dialect-specs.md) | Dialect specs | `nuclear` | Accepted |
| [0026](../docs/adr/0026-enum-item-without-value.md) | Enum item without a value | `nuclear` | Accepted |
| [0027](../docs/adr/0027-cast-non-null-generic-call.md) | `x as T`, `x!`, `X<T>(...)` | `nuclear` | Accepted |
| [0028](../docs/adr/0028-bounded-generics.md) | Bounded generics | `nuclear` | Accepted |
| [0029](../docs/adr/0029-generators.md) | Generators | `nuclear` | Accepted |
| [0030](../docs/adr/0030-cascades.md) | Cascades | `nuclear` | Accepted |
| [0031](../docs/adr/0031-no-records.md) | No records | `nuclear` | Accepted |
| [0032](../docs/adr/0032-destructuring.md) | Destructuring | `nuclear` | Accepted |
| [0033](../docs/adr/0033-if-in-lists.md) | `if` inside lists | `nuclear` | Accepted |
| [0034](../docs/adr/0034-inheriting-constructors.md) | Constructor of an inheriting module | `nuclear` | Accepted |
| [0035](../docs/adr/0035-external-classes.md) | External classes | `nuclear` | Accepted |
| [0036](../docs/adr/0036-results.md) | Results | `nuclear` | Accepted; order amended by ADR-0047 |
| [0037](../docs/adr/0037-function-aliases.md) | Function aliases | `nuclear` | Accepted |
| [0038](../docs/adr/0038-golden-rule.md) | Golden rule | `nuclear` | Accepted |
| [0039](../docs/adr/0039-flutter-dialect-as-framework.md) | The `flutter` dialect as a framework | `dragonfly-flutter` | Accepted |
| [0040](../docs/adr/0040-type-definitions.md) | Cross-dialect type definitions | `nuclear` | Accepted |
| [0041](../docs/adr/0041-dragonfly.md) | Dragonfly | `dragonfly` | Accepted; generalized by ADR-0072 |
| [0042](../docs/adr/0042-component-structure.md) | Component structure | `dragonfly` | Accepted |
| [0043](../docs/adr/0043-dependency-injection.md) | Dependency injection | `dragonfly` | Accepted |
| [0044](../docs/adr/0044-json.md) | JSON | `dragonfly-flutter` | Accepted |
| [0045](../docs/adr/0045-separate-project.md) | Separate project | `dragonfly` | Accepted |
| [0046](../docs/adr/0046-architecture-rules.md) | Architecture rules (linter) | `dragonfly` | Accepted |
| [0047](../docs/adr/0047-result-order-dart-runtime.md) | `Result` order and Dart runtime | `nuclear` | Accepted |
| [0048](../docs/adr/0048-ok-error-by-callback-return.md) | `.ok()` / `.error()` by what the callback returns | `nuclear` | Accepted |
| [0049](../docs/adr/0049-tests-in-nuclear.md) | Tests in Nuclear | `dragonfly-flutter` | Accepted |
| [0050](../docs/adr/0050-nuc-only-project.md) | A Dragonfly project is only `.nuc` | `dragonfly` | Accepted |
| [0051](../docs/adr/0051-dragonfly-package-and-dialect.md) | Dragonfly package and `flutter` dialect | `dragonfly-flutter` | Accepted |
| [0052](../docs/adr/0052-global-events.md) | Global events | `dragonfly` | Accepted |
| [0053](../docs/adr/0053-global-components.md) | Global components | `dragonfly` | Accepted |
| [0054](../docs/adr/0054-session.md) | Session | `dragonfly` | Accepted |
| [0055](../docs/adr/0055-screen-decorator.md) | `@screen` | `dragonfly-flutter` | Accepted |
| [0056](../docs/adr/0056-acl.md) | ACL | `dragonfly` | Accepted |
| [0057](../docs/adr/0057-routes.md) | Routes | `dragonfly-flutter` | Accepted |
| [0058](../docs/adr/0058-forms.md) | Forms | `dragonfly` | Accepted |
| [0059](../docs/adr/0059-dialogs.md) | Dialogs | `dragonfly-flutter` | Accepted |
| [0060](../docs/adr/0060-translations.md) | Translations | `dragonfly` | Accepted |
| [0061](../docs/adr/0061-splash.md) | Splash | `dragonfly-flutter` | Accepted |
| [0062](../docs/adr/0062-generic-models.md) | Generic models | `dragonfly` | Accepted |
| [0063](../docs/adr/0063-native-splash.md) | Native splash | `dragonfly-flutter` | Accepted |
| [0064](../docs/adr/0064-json-names-and-converters.md) | JSON names and converters | `dragonfly-flutter` | Accepted |
| [0065](../docs/adr/0065-manifest-in-nuclear.md) | Manifest in Nuclear (`config.nuc`) | `dragonfly-flutter` | Accepted |
| [0066](../docs/adr/0066-third-party-packages-defs.md) | Third-party packages with definitions | `dragonfly-flutter` | Accepted |
| [0067](../docs/adr/0067-commands-per-dialect.md) | Commands per dialect | `dragonfly` | Accepted |
| [0068](../docs/adr/0068-debug-nuc.md) | Debugging `.nuc` | `editors` | Accepted |
| [0069](../docs/adr/0069-only-third-party-packages.md) | Only third-party packages | `dragonfly-flutter` | Accepted |
| [0070](../docs/adr/0070-no-empty-config-sections.md) | No empty sections in `config.nuc` | `dragonfly-flutter` | Accepted |
| [0071](../docs/adr/0071-assets-by-convention.md) | Assets by convention | `dragonfly-flutter` | Accepted |
| [0072](../docs/adr/0072-dragonfly-language-independent.md) | Dragonfly is language independent | `dragonfly` | Accepted |
| [0073](../docs/adr/0073-dialects-with-agents.md) | New dialects with agents | `nuclear` | Accepted |
| [0074](../docs/adr/0074-dragonfly-device.md) | Dragonfly device | `editors` | Accepted |
| [0075](../docs/adr/0075-code-in-english.md) | Code in English, documentation in Spanish | `nuclear` | Superseded in part by ADR-0082 |
| [0076](../docs/adr/0076-mixed-debugging.md) | Mixed debugging | `editors` | Accepted |
| [0077](../docs/adr/0077-editor-extensions-per-framework.md) | Editor extensions per framework | `editors` | Accepted |
| [0078](../docs/adr/0078-component-logs.md) | Readable logs by component | `dragonfly-flutter` | Accepted |
| [0079](../docs/adr/0079-editor-documentation.md) | Editor documentation and doc comments | `nuclear` | Accepted |
| [0080](../docs/adr/0080-comment-lines-with-star.md) | Comment lines with ` * ` | `nuclear` | Accepted |
| [0081](../docs/adr/0081-cli-devices.md) | Dragonfly devices from the CLI | `dragonfly` | Accepted |
| [0082](../docs/adr/0082-documentation-in-english.md) | Documentation in English | `nuclear` | Accepted |
| [0083](../docs/adr/0083-repository-split.md) | Repository split | `nuclear` | Accepted |
| [0084](../docs/adr/0084-layered-ai-context.md) | Layered AI context | `nuclear` | Accepted |
| [0085](../docs/adr/0085-dragonfly-proposals-accepted.md) | Dragonfly proposals accepted | `dragonfly-flutter` | Accepted |
| [0086](../docs/adr/0086-operators-and-the-targets-library.md) | Operators, literals and the target's library | `nuclear` | Accepted |
| [0087](../docs/adr/0087-agent-support.md) | Agent support: brief, skill, agent init, hook, evaluation | `nuclear` | Accepted |
| [0088](../docs/adr/0088-compiler-corrects-agents.md) | The compiler corrects agents: habits, invented APIs, words on demand, llms.txt | `nuclear` | Accepted |
| [0089](../docs/adr/0089-python-dialect.md) | Python dialect and the generic emitter options it needed | `nuclear` | Accepted |
| [0090](../docs/adr/0090-dragonfly-python-langgraph.md) | Dragonfly for Python: the fastapi dialect (FastAPI, SQLAlchemy, pydantic, LangChain, LangGraph) | `dragonfly` | Accepted |
| [0091](../docs/adr/0091-dragonfly-nestjs-sequelize.md) | Dragonfly for NestJS: the nestjs dialect with Sequelize, Kafka, Redis and LangChain.js | `dragonfly` | Accepted |
| [0092](../docs/adr/0092-plugins-and-genkit.md) | Plugins in config.nuc, and Genkit for Dragonfly Flutter | `dragonfly` | Accepted |
| [0093](../docs/adr/0093-platforms-and-dialects.md) | Three platforms (web, mobile, backend); dialects grouped under them, never visible in the Nuclear code | `dragonfly` | Accepted |
| [0094](../docs/adr/0094-standard-library.md) | Nuclear has its own standard library, translated by every dialect | `nuclear` | Accepted |
| [0095](../docs/adr/0095-kinds-as-libraries.md) | Kinds are Nuclear's libraries: native kinds of the platform, extension kinds as packages | `dragonfly` | Accepted |
| [0096](../docs/adr/0096-mobile-local-database.md) | Local database of the mobile platform: `@primary` models, `@repository(saveLocal)` and `<method>AsLocal()` | `dragonfly-mobile` | Accepted |
| [0097](../docs/adr/0097-web-platform-react.md) | The web platform: the mobile platform in a browser, first dialect `react` | `dragonfly-web` | Accepted |
| [0098](../docs/adr/0098-shared-contracts.md) | Shared contracts: a backend publishes what it exposes, a client generates its models and repositories | `dragonfly` | Accepted |
| [0099](../docs/adr/0099-editor-workspace.md) | The Nuclear workspace in the editor: projects as tabs, tools full screen, keyboard first | `editors` | Accepted |
| [0100](../docs/adr/0100-backend-data.md) | Backend data: one key decorator, TypeORM relations, pagination, transactions, migrations | `dragonfly` | Accepted |
| [0101](../docs/adr/0101-toolchain-data-and-installers.md) | Run, debug and install from data: `[toolchain]`, the card, the release installers | `dragonfly` | Accepted |
| [0102](../docs/adr/0102-ai-loop.md) | The AI loop: Nuclear Smart Graph (private RAG + MCP), Nuclear Developer (local 3B agents), the Loop tool | `nuclear` | Accepted |
| [0103](../docs/adr/0103-generators.md) | The CLI and the MCP generate the code (`dragonfly gen`); the agent writes only what is not standard; HTTP repositories in the backend | `dragonfly` | Accepted |
| [0104](../docs/adr/0104-smart-graph-shelves.md) | The Smart Graph is a library of typed, linked, versioned rows: nine shelves, cards with ids and links, search by meaning and by exact name, assets generated from the tree or written by hand, a revision and a check that never lets the base stay behind the code | `nuclear-graph` | Accepted |
| [0105](../docs/adr/0105-strategies.md) | A strategy for each kind of request (a whole system, one screen, a rename, a failing test...): a row of the graph says how the kind is recognised, its plan with the tool of every step and what it asks of a run; the selection asks no model; the analyst and the Loop follow it; tools before models | `nuclear-graph` | Accepted |
| [0106](../docs/adr/0106-ai-setup.md) | The AI services come with the installer: every release archive carries their code; one set-up (the editor's wizard, `nuclear-loop setup`) reads the machine, suggests a profile of models (`remote`, `lite`, `medium`, `hard`, each with what works and what does not), installs what is missing (uv, Ollama) and starts the Smart Graph and the Nuclear Developer at every login |
