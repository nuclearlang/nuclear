#!/usr/bin/env node
// Type definitions of TypeScript packages (spec/definitions.md), read from their .d.ts with the TypeScript compiler.
//
//   node tools/defs/typescript/gen.js <name> <version> <out-root> <module>... [--depends a@1,b@2] [--lib es2022]
//        [--description "..."] [--from <dir with node_modules>]
//
// Writes <out-root>/typescript/<name>/<version>/{package.toml,index.json,api.json}. `--lib es2022` describes the
// standard library (lib.es2022.d.ts and what it references) as an implicit library: its names never need an import.
// Types are written in NuclearLang notation (number, string, boolean, T[], T?, <K, V>, (a: A) => R).
'use strict';
const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
const opt = (flag, fallback = '') => {
  const i = args.indexOf(flag);
  if (i < 0) return fallback;
  const v = args[i + 1];
  args.splice(i, 2);
  return v;
};
const depends = opt('--depends').split(',').filter(Boolean);
const lib = opt('--lib');
const description = opt('--description');
const from = path.resolve(opt('--from', path.join(__dirname, '../../node')));
const [name, version, outRoot, ...modules] = args;
const ts = require(require.resolve('typescript', { paths: [from] }));

// Bundler resolution reads the `exports` of package.json (subpath entry points: 'dragonfly-nest/ai')
const options = { strict: true, target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler, experimentalDecorators: true, skipLibCheck: true,
  types: [], lib: [`lib.${lib || 'es2022'}.d.ts`] };
const host = ts.createCompilerHost(options);
const probe = path.join(from, '__defs_probe__.ts');

// the declaration file of each module, resolved as the project would import it
const entries = modules.map((m) => {
  if (lib) return null;
  const r = ts.resolveModuleName(m, probe, options, host).resolvedModule;
  if (!r) throw new Error(`cannot resolve the types of '${m}' from ${from}`);
  return r.resolvedFileName;
});
const rootFiles = lib ? [path.join(path.dirname(require.resolve('typescript', { paths: [from] })), `lib.${lib}.d.ts`)] : entries;
const program = ts.createProgram(rootFiles, options, host);
const checker = program.getTypeChecker();

// ---------- types in Nuclear notation ----------
function nuc(t, depth = 0) {
  const s = typeof t === 'string' ? t : checker.typeToString(t, undefined, ts.TypeFormatFlags.NoTruncation | ts.TypeFormatFlags.UseFullyQualifiedType);
  return text(s, depth);
}
function splitTop(s, sep) {
  const out = []; let depth = 0; let cur = '';
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if ('<([{'.includes(c)) depth++;
    else if ('>)]}'.includes(c) && !(c === '>' && s[i - 1] === '=')) depth--;
    if (depth === 0 && s.startsWith(sep, i)) { out.push(cur.trim()); cur = ''; i += sep.length - 1; continue; }
    cur += c;
  }
  out.push(cur.trim());
  return out;
}
function text(s, depth) {
  s = s.trim().replace(/^import\("[^"]*"\)\./, '');
  if (depth > 8 || !s) return 'any';
  const alts = splitTop(s, ' | ');
  if (alts.length > 1) {
    const rest = alts.filter((a) => a !== 'undefined' && a !== 'null');
    const inner = rest.map((a) => text(a, depth + 1)).join(' | ') || 'any';
    return rest.length < alts.length ? (rest.length > 1 ? `(${inner})?` : `${inner}?`) : inner;
  }
  if (s.startsWith('(') && s.endsWith(')') && splitTop(s.slice(1, -1), ' | ').length > 0 && !s.includes('=>')) return text(s.slice(1, -1), depth + 1);
  const fn = s.match(/^\((.*)\) => (.*)$/s);
  if (fn) {
    const ps = splitTop(fn[1], ', ').filter(Boolean).map((p, i) => {
      const [n, ...t] = p.replace(/^\.\.\./, '').split(':');
      return `${n.replace('?', '').trim() || 'a' + i}: ${t.length ? text(t.join(':'), depth + 1) : 'any'}`;
    });
    return `(${ps.join(', ')}) => ${text(fn[2], depth + 1)}`;
  }
  if (s.endsWith('[]')) return `${text(s.slice(0, -2), depth + 1)}[]`;
  const g = s.match(/^([\w.$]+)<(.*)>$/s);
  if (g) {
    const a = splitTop(g[2], ', ');
    if (g[1] === 'Array' || g[1] === 'ReadonlyArray') return `${text(a[0], depth + 1)}[]`;
    if (g[1] === 'Record' && a.length === 2) return `<${text(a[0], depth + 1)}, ${text(a[1], depth + 1)}>`;
    return `${g[1].split('.').pop()}<${a.map((x) => text(x, depth + 1)).join(', ')}>`;
  }
  if (/^[\w.$]+$/.test(s)) return s.split('.').pop() === 'undefined' ? 'void' : s.split('.').pop();
  if (/^["'].*["']$/.test(s) || /^-?\d/.test(s)) return s;
  return 'any';
}

// ---------- declarations ----------
const isPublic = (n) => n && !n.startsWith('_') && !n.startsWith('#') && !n.startsWith('__@');
const docOf = (sym) => {
  const d = ts.displayPartsToString(sym.getDocumentationComment(checker)).split('\n\n')[0].replace(/\s+/g, ' ').trim();
  return d ? d.slice(0, 200) : undefined;
};
const deprecated = (sym) => sym.getJsDocTags().some((t) => t.name === 'deprecated');
const decl0 = (sym) => (sym.declarations || [])[0];

function params(sig) {
  return sig.parameters.map((p) => {
    const d = decl0(p);
    const t = checker.getTypeOfSymbolAtLocation(p, d || sig.declaration);
    const out = { name: (d && d.dotDotDotToken ? '...' : '') + p.name, type: nuc(t) };
    if (d && (d.questionToken || d.initializer || d.dotDotDotToken)) out.optional = true;
    return out;
  });
}
function members(type, own) {
  const out = [];
  for (const p of checker.getPropertiesOfType(type)) {
    if (!isPublic(p.name)) continue;
    const d = decl0(p);
    if (own && d && d.parent && own !== d.parent && !(own.members || []).includes(d)) continue;
    const t = checker.getTypeOfSymbolAtLocation(p, d || own);
    const sigs = t.getCallSignatures();
    const m = { name: p.name };
    if (p.flags & ts.SymbolFlags.Method && sigs.length) {
      m.kind = 'method';
      m.params = params(sigs[0]);
      m.returns = nuc(sigs[0].getReturnType());
    } else {
      m.kind = 'property';
      m.type = nuc(t);
      if (d && ts.getCombinedModifierFlags(d) & ts.ModifierFlags.Readonly) m.readonly = true;
    }
    const doc = docOf(p);
    if (doc) m.doc = doc;
    if (deprecated(p)) m.deprecated = true;
    out.push(m);
  }
  return out;
}
function staticMembers(ctorType) {
  return members(ctorType).filter((m) => !['prototype'].includes(m.name)).map((m) => ({ ...m, static: true }));
}

function declaration(exportName, sym, library) {
  if (sym.flags & ts.SymbolFlags.Alias) sym = checker.getAliasedSymbol(sym);
  const d = decl0(sym);
  if (!d) return null;
  const out = { name: exportName, library };
  const doc = docOf(sym);
  if (sym.flags & ts.SymbolFlags.Class) {
    const inst = checker.getDeclaredTypeOfSymbol(sym);
    const stat = checker.getTypeOfSymbolAtLocation(sym, d);
    out.kind = 'class';
    const bases = (inst.isClassOrInterface() && checker.getBaseTypes(inst)) || [];
    if (bases[0]) out.extends = bases[0].symbol ? bases[0].symbol.name : nuc(bases[0]);
    const impl = (d.heritageClauses || []).filter((h) => h.token === ts.SyntaxKind.ImplementsKeyword)
      .flatMap((h) => h.types.map((t) => t.expression.getText()));
    if (impl.length) out.implements = impl;
    const tp = inst.typeParameters || [];
    if (tp.length) out.type_params = tp.map((t) => t.symbol.name);
    if (ts.getCombinedModifierFlags(d) & ts.ModifierFlags.Abstract) out.modifiers = ['abstract'];
    const ctors = stat.getConstructSignatures();
    if (ctors.length) out.constructors = [{ name: '', params: params(ctors[0]) }];
    out.members = [...members(inst), ...staticMembers(stat)];
  } else if (sym.flags & ts.SymbolFlags.Interface) {
    const inst = checker.getDeclaredTypeOfSymbol(sym);
    out.kind = 'class';
    out.modifiers = ['interface'];
    const bases = checker.getBaseTypes(inst) || [];
    if (bases.length) out.implements = bases.map((b) => (b.symbol ? b.symbol.name : nuc(b)));
    const tp = inst.typeParameters || [];
    if (tp.length) out.type_params = tp.map((t) => t.symbol.name);
    out.members = members(inst);
    // `declare var Date: DateConstructor` (the standard library): the interface is a class you can construct
    if (sym.flags & ts.SymbolFlags.Variable) {
      const vt = checker.getTypeOfSymbolAtLocation(sym, (sym.declarations || []).find(ts.isVariableDeclaration) || d);
      const ctors = vt.getConstructSignatures();
      if (ctors.length) out.constructors = [{ name: '', params: params(ctors[0]) }];
      delete out.modifiers;
      out.members.push(...staticMembers(vt));
    }
  } else if (sym.flags & ts.SymbolFlags.Enum) {
    out.kind = 'enum';
    out.values = checker.getExportsOfModule(sym).map((e) => e.name);
  } else if (sym.flags & ts.SymbolFlags.Function) {
    const sig = checker.getTypeOfSymbolAtLocation(sym, d).getCallSignatures()[0];
    out.kind = 'function';
    if (sig) {
      out.params = params(sig);
      out.returns = nuc(sig.getReturnType());
    }
  } else if (sym.flags & ts.SymbolFlags.Variable) {
    const t = checker.getTypeOfSymbolAtLocation(sym, d);
    const sigs = t.getCallSignatures();
    if (sigs.length) {
      out.kind = 'function';
      out.params = params(sigs[0]);
      out.returns = nuc(sigs[0].getReturnType());
    } else {
      out.kind = 'variable';
      out.type = nuc(t);
    }
  } else if (sym.flags & ts.SymbolFlags.TypeAlias) {
    out.kind = 'typedef';
    out.type = nuc(checker.getDeclaredTypeOfSymbol(sym));
  } else {
    return null;
  }
  if (doc) out.doc = doc;
  if (deprecated(sym)) out.deprecated = true;
  return out;
}

const decls = new Map();
const symbols = {};
const libraries = lib ? [`lib.${lib}`] : modules;
if (lib) {
  // the standard library: the global names its files declare
  const globals = checker.getSymbolsInScope(program.getSourceFile(rootFiles[0]), ts.SymbolFlags.Type | ts.SymbolFlags.Value);
  for (const s of globals) {
    const fromLib = (s.declarations || []).some((d) => /lib\.[\w.]+\.d\.ts$/.test(d.getSourceFile().fileName));
    if (!fromLib || !isPublic(s.name) || decls.has(s.name)) continue;
    const d = declaration(s.name, s, libraries[0]);
    if (d) { decls.set(s.name, d); symbols[s.name] = 0; }
  }
} else {
  entries.forEach((file, li) => {
    const sf = program.getSourceFile(file);
    const ms = sf && checker.getSymbolAtLocation(sf);
    if (!ms) return;
    for (const e of checker.getExportsOfModule(ms)) {
      if (!isPublic(e.name) || e.name === 'default' || decls.has(e.name)) continue;
      const d = declaration(e.name, e, modules[li]);
      if (d) { decls.set(e.name, d); symbols[e.name] = li; }
    }
  });
}

// index.json: what the transpiler reads on every build
const classes = {};
for (const [n, d] of decls) {
  if (d.kind !== 'class') continue;
  const entry = { supers: [d.extends, ...(d.implements || [])].filter(Boolean),
    members: [...new Set((d.members || []).filter((m) => !m.static).map((m) => m.name))].sort() };
  if (d.modifiers && d.modifiers.includes('interface')) entry.kind = 'interface';
  const results = {};
  for (const m of d.members || []) {
    if (m.kind !== 'method' || !m.returns) continue;
    const r = m.returns.replace(/^Promise<(.*)>$/, '$1');
    if (r.startsWith('Result<')) results[m.name] = r;
  }
  if (Object.keys(results).length) entry.results = results;
  if (d.constructors && d.constructors[0]) entry.params = d.constructors[0].params.map((p) => p.name).filter((p) => !p.startsWith('...'));
  classes[n] = entry;
}
const out = path.join(outRoot, 'typescript', name, version);
fs.mkdirSync(out, { recursive: true });
const toml = [
  '# Generated by tools/defs/typescript/gen.js. Do not edit by hand: regenerate it (tools/defs/typescript/generate.sh).',
  '[package]', `name = "${name}"`, `version = "${version}"`, 'language = "typescript"',
  `description = ${JSON.stringify(description || name)}`,
  `depends = [${depends.map((x) => JSON.stringify(x)).join(', ')}]`, 'libraries = [',
  ...libraries.map((l) => `  "${l}",`), ']', ...(lib ? [`implicit = ["${libraries[0]}"]`] : []), '',
];
fs.writeFileSync(path.join(out, 'package.toml'), toml.join('\n'));
const entriesOf = (m) => Object.keys(m).sort().map((k) => `  ${JSON.stringify(k)}: ${JSON.stringify(m[k])}`).join(',\n');
fs.writeFileSync(path.join(out, 'index.json'),
  `{\n"format": 1,\n"libraries": ${JSON.stringify(libraries)},\n"reexports": {},\n"symbols": {\n${entriesOf(symbols)}\n},\n"classes": {\n${entriesOf(classes)}\n}\n}\n`);
fs.writeFileSync(path.join(out, 'api.json'),
  `{"format": 1, "declarations": [\n${[...decls.keys()].sort().map((k) => JSON.stringify(decls.get(k))).join(',\n')}\n]}\n`);
console.log(`${out}: ${Object.keys(symbols).length} symbols, ${Object.keys(classes).length} classes`);
