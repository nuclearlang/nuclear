// Generates a NuclearLang type definitions package (like TypeScript's .d.ts / @types packages) from the
// public API of Dart libraries, using the official analyzer.
//
//   dart run bin/defs.dart --project <dir> --out <package dir> --name <n> --version <v>
//       [--description <text>] [--depends <name@version>]... [--implicit <library uri>]...
//       [--exclude <package dir>]... <library uri>...
//
// Writes three files in --out (format: spec/definitions.md):
//   package.toml  manifest: name, version, language, libraries (in priority order), depends
//   index.json    what the transpiler reads on every build: symbol → library, re-exports and, for each
//                 class, its kind, direct supertypes and own instance member names
//   api.json      the full API, one declaration per line, read lazily by tools (LSP, `nuclear defs`):
//                 constructors, parameters, types, members, enum values, extensions, functions, docs
//
// Types are written in NuclearLang notation (`string`, `int[]`, `<string, int>`, `(x: int) => void`),
// so every tool reads the same notation whatever the target language is.
// --exclude names packages already generated (the dependencies): their declarations are not repeated.
// --implicit marks libraries that are always visible (dart:core): their names are never auto-imported.

import 'dart:convert';
import 'dart:io';

import 'package:analyzer/dart/analysis/analysis_context_collection.dart';
import 'package:analyzer/dart/analysis/results.dart';
import 'package:analyzer/dart/element/element.dart';
import 'package:analyzer/dart/element/nullability_suffix.dart';
import 'package:analyzer/dart/element/type.dart';

final identifier = RegExp(r'^[A-Za-z_][A-Za-z0-9_]*$');
final symbolName = RegExp(r'^[A-Za-z][A-Za-z0-9_]*$');

Future<void> main(List<String> args) async {
  String? project, out, name, version, description;
  final depends = <String>[];
  final implicit = <String>[];
  final excludeDirs = <String>[];
  final uris = <String>[];
  for (var i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--project':
        project = args[++i];
      case '--out':
        out = args[++i];
      case '--name':
        name = args[++i];
      case '--version':
        version = args[++i];
      case '--description':
        description = args[++i];
      case '--depends':
        depends.add(args[++i]);
      case '--implicit':
        implicit.add(args[++i]);
      case '--exclude':
        excludeDirs.add(args[++i]);
      default:
        uris.add(args[i]);
    }
  }
  if (project == null || out == null || name == null || version == null || uris.isEmpty) {
    stderr.writeln('usage: defs.dart --project <dir> --out <dir> --name <n> --version <v> '
        '[--description <t>] [--depends <n@v>]... [--implicit <uri>]... [--exclude <dir>]... <library uri>...');
    exit(64);
  }

  final root = Directory(project).absolute.uri.normalizePath().toFilePath().replaceAll(RegExp(r'/$'), '');
  final session = AnalysisContextCollection(includedPaths: [root]).contextFor(root).currentSession;

  Future<LibraryElement> library(String uri) async {
    final result = await session.getLibraryByUri(uri);
    if (result is! LibraryElementResult) {
      stderr.writeln('cannot resolve $uri (${result.runtimeType})');
      exit(1);
    }
    return result.element;
  }

  // declarations of the dependencies are not repeated; their libraries count for re-exports
  final skip = <String>{};
  final depLibraries = <String>[];
  for (final dir in excludeDirs) {
    final index = jsonDecode(File('$dir/index.json').readAsStringSync()) as Map<String, dynamic>;
    skip.addAll((index['classes'] as Map<String, dynamic>).keys);
    for (final line in File('$dir/api.json').readAsLinesSync()) {
      final m = RegExp(r'^\{"name":"([^"]+)"').firstMatch(line);
      if (m != null) skip.add(m[1]!);
    }
    depLibraries.addAll((index['libraries'] as List).cast<String>());
  }

  final libs = {for (final uri in uris) uri: await library(uri)};
  Map<String, Element> exported(LibraryElement l) => {
        for (final e in l.exportNamespace.definedNames2.entries)
          if (!e.key.endsWith('=')) e.key: e.value,
      };
  final namespaces = {for (final uri in uris) uri: exported(libs[uri]!)};

  // symbol → first library (in the given order) that exports it
  final symbols = <String, int>{};
  for (var i = 0; i < uris.length; i++) {
    for (final n in namespaces[uris[i]]!.keys) {
      if (symbolName.hasMatch(n)) symbols.putIfAbsent(n, () => i);
    }
  }

  // re-exports: library A includes every name of library B (B of this package or of a dependency)
  final reexports = <String, List<String>>{};
  final depNamespaces = {for (final uri in depLibraries) uri: exported(await library(uri)).keys.toSet()};
  final allNames = {for (final uri in uris) uri: namespaces[uri]!.keys.toSet(), ...depNamespaces};
  for (final a in uris) {
    for (final b in [...uris, ...depLibraries]) {
      final na = allNames[a]!, nb = allNames[b]!;
      if (a != b && nb.isNotEmpty && na.containsAll(nb) && na.length > nb.length) {
        (reexports[a] ??= []).add(b);
      }
    }
  }

  // declarations: everything the libraries export, plus the private supertypes of their classes
  final decls = <String, Map<String, Object?>>{};
  final index = <String, Map<String, Object?>>{};
  final pending = <(Element, String?)>[];
  for (final uri in uris) {
    for (final e in namespaces[uri]!.values) {
      pending.add((e, uri));
    }
  }
  while (pending.isNotEmpty) {
    final (e, uri) = pending.removeLast();
    final n = e.name;
    if (n == null || decls.containsKey(n) || skip.contains(n)) continue;
    final lib = n.startsWith('_') ? null : uris.where((u) => namespaces[u]![n] == e).firstOrNull;
    final decl = declaration(e, lib);
    if (decl == null) continue;
    decls[n] = decl;
    if (e is InterfaceElement) {
      final supers = <String>[];
      for (final t in [
        if (e.supertype != null) e.supertype!,
        ...e.mixins,
        ...e.interfaces,
        if (e is MixinElement) ...e.superclassConstraints,
      ]) {
        final s = t.element.name;
        if (s == null || supers.contains(s)) continue;
        supers.add(s);
        pending.add((t.element, null));
      }
      final members = <String>{
        for (final m in e.methods) if (!m.isStatic) m.name ?? '',
        for (final g in e.getters) if (!g.isStatic) g.name ?? '',
        for (final s in e.setters) if (!s.isStatic) s.name ?? '',
        for (final f in e.fields) if (!f.isStatic) f.name ?? '',
      }.where((x) => identifier.hasMatch(x) && !x.startsWith('_')).toList()
        ..sort();
      final kind = e is MixinElement ? 'mixin' : (e is ClassElement && e.isInterface ? 'interface' : null);
      // the parameters of the unnamed constructor: the transpiler rejects a named argument that is not one
      final ctor = (decl['constructors'] as List?)?.cast<Map<String, Object?>>().where((c) => c['name'] == '').firstOrNull;
      index[n] = {
        if (kind != null) 'kind': kind,
        'supers': supers,
        'members': members,
        if (ctor != null) 'params': [for (final p in ctor['params'] as List) (p as Map)['name']],
      };
    }
  }

  Directory(out).createSync(recursive: true);
  final language = 'dart';
  final toml = StringBuffer()
    ..writeln('# Generated by tools/defs (bin/defs.dart). Do not edit by hand: regenerate it (see tools/defs/README.md).')
    ..writeln('[package]')
    ..writeln('name = "$name"')
    ..writeln('version = "$version"')
    ..writeln('language = "$language"');
  if (description != null) toml.writeln('description = ${jsonEncode(description)}');
  toml.writeln('depends = [${depends.map(jsonEncode).join(', ')}]');
  toml.writeln('libraries = [');
  for (final u in uris) {
    toml.writeln('  "$u",');
  }
  toml.writeln(']');
  if (implicit.isNotEmpty) toml.writeln('implicit = [${implicit.map(jsonEncode).join(', ')}]');
  File('$out/package.toml').writeAsStringSync(toml.toString());

  // index.json: one entry per line, sorted, so diffs between versions stay readable
  String entries(Map<String, Object?> m) {
    final keys = m.keys.toList()..sort();
    return keys.map((k) => '  ${jsonEncode(k)}: ${jsonEncode(m[k])}').join(',\n');
  }

  final idx = StringBuffer()
    ..writeln('{')
    ..writeln('"format": 1,')
    ..writeln('"libraries": ${jsonEncode(uris)},')
    ..writeln('"reexports": ${jsonEncode(reexports)},')
    ..writeln('"symbols": {')
    ..writeln(entries(symbols))
    ..writeln('},')
    ..writeln('"classes": {')
    ..writeln(entries(index))
    ..writeln('}')
    ..writeln('}');
  File('$out/index.json').writeAsStringSync(idx.toString());

  final names = decls.keys.toList()..sort();
  final api = StringBuffer()
    ..writeln('{"format": 1, "declarations": [')
    ..writeln(names.map((n) => jsonEncode(decls[n])).join(',\n'))
    ..writeln(']}');
  File('$out/api.json').writeAsStringSync(api.toString());
  stdout.writeln('$out: ${symbols.length} symbols, ${names.length} declarations, ${index.length} classes');
}

/// One declaration of api.json, or null for elements that are not declarations (prefixes...).
Map<String, Object?>? declaration(Element e, String? library) {
  final base = <String, Object?>{'name': e.name, if (library != null) 'library': library};
  if (e is InterfaceElement) {
    final kind = switch (e) {
      EnumElement() => 'enum',
      MixinElement() => 'mixin',
      ExtensionTypeElement() => 'extension_type',
      _ => 'class',
    };
    final modifiers = <String>[
      if (e is ClassElement && e.isAbstract) 'abstract',
      if (e is ClassElement && e.isSealed) 'sealed',
      if (e is ClassElement && e.isBase) 'base',
      if (e is ClassElement && e.isInterface) 'interface',
      if (e is ClassElement && e.isFinal) 'final',
      if (e is ClassElement && e.isMixinClass) 'mixin',
      if (e is MixinElement && e.isBase) 'base',
    ];
    return {
      ...base,
      'kind': kind,
      if (modifiers.isNotEmpty) 'modifiers': modifiers,
      ...typeParams(e.typeParameters),
      if (e.supertype != null && !e.supertype!.isDartCoreObject && e is! EnumElement) 'extends': type(e.supertype!),
      if (e.mixins.isNotEmpty) 'with': e.mixins.map(type).toList(),
      if (e.interfaces.isNotEmpty) 'implements': e.interfaces.map(type).toList(),
      if (e is MixinElement && e.superclassConstraints.any((t) => !t.isDartCoreObject))
        'on': e.superclassConstraints.map(type).toList(),
      ...docOf(e),
      if (e is EnumElement) 'values': e.constants.map((c) => c.name).whereType<String>().toList(),
      if (e is! EnumElement && e is! MixinElement) 'constructors': constructors(e),
      'members': members(e, skipEnum: e is EnumElement),
    }..removeWhere((k, v) => k == 'constructors' && (v as List).isEmpty);
  }
  if (e is ExtensionElement) {
    return {...base, 'kind': 'extension', ...typeParams(e.typeParameters), 'on': [type(e.extendedType)], ...docOf(e), 'members': members(e)};
  }
  if (e is TopLevelFunctionElement) {
    return {...base, 'kind': 'function', ...signature(e), ...docOf(e)};
  }
  if (e is GetterElement && e.variable is TopLevelVariableElement) {
    final v = e.variable as TopLevelVariableElement;
    return {
      ...base,
      'kind': 'variable',
      'type': type(v.type),
      if (v.isConst || v.isFinal || v.setter == null) 'readonly': true,
      ...docOf(v),
    };
  }
  if (e is TopLevelVariableElement) {
    return {...base, 'kind': 'variable', 'type': type(e.type), if (e.isConst || e.isFinal) 'readonly': true, ...docOf(e)};
  }
  if (e is TypeAliasElement) {
    return {...base, 'kind': 'typedef', ...typeParams(e.typeParameters), 'type': type(e.aliasedType, expandAlias: true), ...docOf(e)};
  }
  return null;
}

List<Map<String, Object?>> constructors(InterfaceElement e) => [
      for (final c in e.constructors)
        if (!(c.name ?? '').startsWith('_'))
          {
            'name': (c.name == null || c.name == 'new') ? '' : c.name,
            if (c.isConst) 'const': true,
            if (c.isFactory) 'factory': true,
            'params': params(c.formalParameters),
            ...docOf(c),
          },
    ];

List<Map<String, Object?>> members(InstanceElement e, {bool skipEnum = false}) {
  final out = <Map<String, Object?>>[];
  final seen = <String>{};
  bool public(String? n) => n != null && !n.startsWith('_') && !(skipEnum && (n == 'values' || n == 'index'));
  for (final f in e.fields) {
    if (f.isOriginGetterSetter || f.isOriginEnumValues || !public(f.name) || (skipEnum && f.isEnumConstant)) continue;
    seen.add(f.name!);
    out.add({
      'name': f.name,
      'kind': 'property',
      if (f.isStatic) 'static': true,
      'type': type(f.type),
      if (f.isFinal || f.isConst) 'readonly': true,
      ...docOf(f),
    });
  }
  for (final g in e.getters) {
    final n = g.name;
    if (!g.isOriginDeclaration || !public(n) || seen.contains(n)) continue;
    seen.add(n!);
    final hasSetter = e.setters.any((s) => s.name == n);
    out.add({
      'name': n,
      'kind': 'property',
      if (g.isStatic) 'static': true,
      'type': type(g.returnType),
      if (!hasSetter) 'readonly': true,
      ...docOf(g),
    });
  }
  for (final s in e.setters) {
    final n = s.name;
    if (!s.isOriginDeclaration || !public(n) || seen.contains(n)) continue;
    seen.add(n!);
    final p = s.formalParameters;
    out.add({
      'name': n,
      'kind': 'property',
      if (s.isStatic) 'static': true,
      'type': p.isEmpty ? 'dynamic' : type(p.first.type),
      ...docOf(s),
    });
  }
  for (final m in e.methods) {
    final n = m.name;
    if (!public(n)) continue;
    out.add({
      'name': n,
      'kind': identifier.hasMatch(n!) ? 'method' : 'operator',
      if (m.isStatic) 'static': true,
      ...signature(m),
      ...docOf(m),
    });
  }
  return out;
}

Map<String, Object?> signature(ExecutableElement e) => {
      ...typeParams(e.typeParameters),
      'params': params(e.formalParameters),
      'returns': type(e.returnType),
    };

List<Map<String, Object?>> params(List<FormalParameterElement> ps) => [
      for (final (i, p) in ps.indexed)
        {
          'name': (p.name == null || p.name!.isEmpty) ? 'arg${i + 1}' : p.name,
          'type': type(p.type),
          if (p.isNamed) 'named': true,
          if (p.isOptional) 'optional': true,
          if (p.defaultValueCode != null) 'default': p.defaultValueCode!.replaceFirst(RegExp(r'^const '), ''),
        },
    ];

Map<String, Object?> typeParams(List<TypeParameterElement> tps) => tps.isEmpty
    ? {}
    : {
        'type_params': [
          for (final t in tps) t.bound == null ? t.name : '${t.name}: ${type(t.bound!)}',
        ],
      };

/// First paragraph of the dartdoc comment, without markdown links, plus the deprecation flag.
Map<String, Object?> docOf(Element e) {
  final out = <String, Object?>{};
  final raw = e.documentationComment;
  if (raw != null) {
    final lines = <String>[];
    for (final l in raw.split('\n')) {
      final t = l.trim().replaceFirst(RegExp(r'^(///?|\*+/?|/\*\*)\s?'), '').trim();
      if (t.isEmpty) {
        if (lines.isNotEmpty) break;
        continue;
      }
      if (t.startsWith('{@') || t.startsWith('```')) break;
      lines.add(t);
    }
    var doc = lines.join(' ').replaceAllMapped(RegExp(r'\[([^\]]+)\](\([^)]*\))?'), (m) => m[1]!);
    if (doc.length > 280) {
      final cut = doc.indexOf('. ', 80);
      doc = cut > 0 && cut < 280 ? doc.substring(0, cut + 1) : '${doc.substring(0, 277)}...';
    }
    if (doc.isNotEmpty) out['doc'] = doc;
  }
  if (e.metadata.hasDeprecated) out['deprecated'] = true;
  return out;
}

/// A Dart type in NuclearLang notation.
String type(DartType t, {bool expandAlias = false}) {
  final q = t.nullabilitySuffix == NullabilitySuffix.question ? '?' : '';
  final alias = t.alias;
  if (alias != null && !expandAlias && !(alias.element.name ?? '_').startsWith('_')) {
    final args = alias.typeArguments.isEmpty ? '' : '<${alias.typeArguments.map(type).join(', ')}>';
    return '${alias.element.name}$args$q';
  }
  if (t is DynamicType || t is InvalidType) return 'dynamic';
  if (t is VoidType) return 'void';
  if (t is NeverType) return 'Never$q';
  if (t is TypeParameterType) return '${t.element.name}$q';
  if (t is FunctionType) {
    final ps = <String>[];
    final named = <String>[];
    for (final (i, p) in t.formalParameters.indexed) {
      final n = (p.name == null || p.name!.isEmpty) ? 'arg${i + 1}' : p.name!;
      final s = p.isOptional && !p.isNamed ? '$n?: ${type(p.type)}' : '$n: ${type(p.type)}';
      (p.isNamed ? named : ps).add(p.isNamed && p.isOptional ? '$n?: ${type(p.type)}' : s);
    }
    if (named.isNotEmpty) ps.add('{${named.join(', ')}}');
    final generics = t.typeParameters.isEmpty ? '' : '<${t.typeParameters.map((p) => p.name).join(', ')}>';
    final f = '$generics(${ps.join(', ')}) => ${type(t.returnType)}';
    return q.isEmpty ? f : '($f)?';
  }
  if (t is RecordType) {
    final fields = [
      ...t.positionalFields.map((f) => type(f.type)),
      if (t.namedFields.isNotEmpty) '{${t.namedFields.map((f) => '${f.name}: ${type(f.type)}').join(', ')}}',
    ];
    return '(${fields.join(', ')})$q';
  }
  if (t is InterfaceType) {
    final n = t.element.name ?? 'Object';
    final args = t.typeArguments;
    if (t.isDartCoreString) return 'string$q';
    if (t.isDartCoreBool) return 'boolean$q';
    if (t.isDartCoreNum) return 'number$q';
    if (t.isDartCoreList && args.length == 1) {
      final item = type(args.first);
      final wrapped = item.contains('=>') ? '($item)' : item;
      return '$wrapped[]$q';
    }
    if (t.isDartCoreMap && args.length == 2) return '<${type(args[0])}, ${type(args[1])}>$q';
    final a = args.isEmpty ? '' : '<${args.map(type).join(', ')}>';
    return '$n$a$q';
  }
  return t.getDisplayString();
}
