#!/usr/bin/env bash
# Regenerates the type definitions packages of the dart language (defs/dart/: the SDK here, the packages in
# dragonfly-mobile/defs/dart/, plan 006) from the
# installed Flutter SDK and the packages of flutter_project/pubspec.yaml. See README.md.
#   FLUTTER_ROOT=~/development/flutter tools/defs/generate.sh
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/../../defs/dart"
# the packages of the flutter dialect live in the mobile platform's repository (plan 006); the Dart SDK here
POUT="${DRAGONFLY_MOBILE:-$HERE/../../dragonfly-mobile}/defs/dart"
FLUTTER_ROOT="${FLUTTER_ROOT:-$HOME/development/flutter}"
DART="$FLUTTER_ROOT/bin/dart"
cd "$HERE"
"$DART" pub get >/dev/null
(cd flutter_project && "$FLUTTER_ROOT/bin/flutter" pub get >/dev/null)

DV=$("$DART" --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
FV=$("$FLUTTER_ROOT/bin/flutter" --version 2>&1 | grep -oE '^Flutter [0-9.]+' | cut -d' ' -f2)
# version of a package resolved in flutter_project
pv() { grep -A8 "^  $1:" flutter_project/pubspec.lock | grep -oE 'version: "[^"]+"' | cut -d'"' -f2; }

gen() { "$DART" run bin/defs.dart "$@"; }

gen --project . --out "$OUT/dart/$DV" --name dart --version "$DV" \
  --description "Dart SDK $DV libraries" --implicit dart:core \
  dart:core dart:async dart:math dart:convert dart:collection dart:io dart:typed_data dart:isolate dart:developer

gen --project flutter_project --out "$POUT/flutter/$FV" --name flutter --version "$FV" \
  --description "Flutter SDK $FV: widgets, material, cupertino, services, foundation, painting, rendering, gestures, animation, scheduler, semantics, physics" \
  --depends "dart@$DV" --exclude "$OUT/dart/$DV" \
  package:flutter/widgets.dart package:flutter/material.dart package:flutter/cupertino.dart \
  package:flutter/services.dart package:flutter/foundation.dart package:flutter/painting.dart \
  package:flutter/rendering.dart package:flutter/gestures.dart package:flutter/animation.dart \
  package:flutter/scheduler.dart package:flutter/semantics.dart package:flutter/physics.dart

# packages from pub.dev (pure Dart first, then the ones that need Flutter)
pub() { # name description library... ; extra flags in $DEPS
  local name=$1 desc=$2; shift 2
  local v; v=$(pv "$name")
  # shellcheck disable=SC2086
  gen --project flutter_project --out "$POUT/$name/$v" --name "$name" --version "$v" --description "$desc" \
    --depends "dart@$DV" --exclude "$OUT/dart/$DV" $DEPS "$@"
}
DEPS="" pub dio "dio: HTTP client (requests, interceptors, cancellation, FormData)" package:dio/dio.dart
DEPS="" pub get_it "get_it: service locator for dependency injection" package:get_it/get_it.dart
DEPS="" pub json_annotation "json_annotation: annotations read by json_serializable" package:json_annotation/json_annotation.dart
DEPS="" pub bloc "bloc: Bloc and Cubit state management" package:bloc/bloc.dart
BV=$(pv bloc)
DEPS="--depends flutter@$FV --depends bloc@$BV --exclude $POUT/flutter/$FV --exclude $POUT/bloc/$BV" \
  pub flutter_bloc "flutter_bloc: widgets for bloc (BlocProvider, BlocBuilder, BlocListener...)" package:flutter_bloc/flutter_bloc.dart
FDEP="--depends flutter@$FV --exclude $POUT/flutter/$FV"
DEPS="$FDEP" pub go_router "go_router: declarative routing (GoRouter, GoRoute, redirects)" package:go_router/go_router.dart
DEPS="$FDEP" pub connectivity_plus "connectivity_plus: connection type (wifi, mobile...) and its changes" package:connectivity_plus/connectivity_plus.dart
DEPS="$FDEP" pub flutter_secure_storage "flutter_secure_storage: Keychain / Keystore storage" package:flutter_secure_storage/flutter_secure_storage.dart

# dragonfly: the framework's runtime (packages/dragonfly), after every package it uses
D=""
for p in flutter_bloc bloc dio get_it go_router connectivity_plus flutter_secure_storage json_annotation; do
  v=$(pv "$p"); D="$D --depends $p@$v --exclude $POUT/$p/$v"
done
DEPS="$FDEP $D" pub dragonfly "dragonfly: runtime of the NuclearLang framework for Flutter (packages/dragonfly)" package:dragonfly/dragonfly.dart package:dragonfly/testing.dart

# the genkit plugin (dialects/mobile/flutter [project.plugins.genkit]): Genkit Dart and the runtime packages/dragonfly_genkit
DEPS="" pub schemantic "schemantic: typed schemas (SchemanticType) of Genkit Dart" package:schemantic/schemantic.dart
SV=$(pv schemantic)
DEPS="--depends schemantic@$SV --exclude $POUT/schemantic/$SV" pub genkit "Genkit Dart: generate, tools, flows, embedders, remote actions" package:genkit/genkit.dart package:genkit/client.dart
GV=$(pv genkit); DFV=$(pv dragonfly)
DEPS="$FDEP --depends genkit@$GV --exclude $POUT/genkit/$GV --depends schemantic@$SV --exclude $POUT/schemantic/$SV --depends dragonfly@$DFV --exclude $POUT/dragonfly/$DFV" \
  pub dragonfly_genkit "dragonfly_genkit: Genkit for Dragonfly apps (packages/dragonfly_genkit)" package:dragonfly_genkit/dragonfly_genkit.dart package:dragonfly_genkit/testing.dart
