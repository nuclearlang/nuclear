# Dialect `nodejs`

> Inherits every rule of [nuclear-base.md](nuclear-base.md) and of the
> [typescript](typescript.md) dialect. This document only defines what `nodejs` adds. Files:
> `dialects/nodejs/`.

## 1. Native modules

The dialect knows the target is Node.js, so native modules are written without `node:`:

| Nuclear | TypeScript |
|---|---|
| `use fs/promises{ readFile, writeFile }` | `import { readFile, writeFile } from 'node:fs/promises';` |
| `use * as path from path` | `import * as path from 'node:path';` |

Shortcuts: `fs`, `fs/promises`, `path`, `os`, `crypto`, `http`, `https`, `events`, `stream`, `url`, `util`,
`child_process`, `worker_threads`, `zlib`... (full list in `[imports]` of
`dialects/nodejs/dialect.toml`).

## 2. Automatic imports

The functions of `node:fs/promises` and `node:fs` are imported when used; no `use` is needed:

```nuclear
loadPrompt(path: string) string
  ~readFile(path, 'utf8')

save(path: string, data: string) void
  ~writeFile(path, data)
  print(process.env.HOME)
```

```typescript
import { readFile, writeFile } from 'node:fs/promises';

export async function loadPrompt(path: string): Promise<string> {
  return await readFile(path, 'utf8');
}

export async function save(path: string, data: string): Promise<void> {
  await writeFile(path, data);
  console.log(process.env.HOME);
}
```

## 3. Global names

`process`, `Buffer`, `__dirname`, `__filename`, `require`, `module`, `exports` y `global` exist without
being declared, besides those of [typescript](typescript.md).

## 4. Complete example

A complete example with this dialect:
[examples/examples-nodejs/chat.nuc](../examples/examples-nodejs/chat.nuc) →
[chat.ts](../examples/examples-nodejs/chat.ts).
