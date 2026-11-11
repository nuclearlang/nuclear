#!/bin/sh
# Regenerates defs/typescript/: the TypeScript standard library and the packages of the nestjs dialect, with the
# versions installed in tools/node (package.json there; packages/dragonfly-nest is linked with
# `npm install --no-save ../../packages/dragonfly-nest`). Then update the `defs` of dialects/backend/nestjs if a version
# changed, and run `cargo test`.
set -e
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
nm="$root/tools/node/node_modules"
v() { node -p "require('$nm/$1/package.json').version"; }
gen() { node "$here/gen.js" "$@"; }
# the packages of the platforms live in their repositories (plan 006); the standard library here
backend="${DRAGONFLY_BACKEND:-$root/dragonfly-backend}/defs"
TS="typescript@$(v typescript)"
gen typescript "$(v typescript)" "$root/defs" --lib es2022 --description "TypeScript standard library (ES2022, implicit)"
gen rxjs "$(v rxjs)" "$backend" rxjs --depends "$TS" --description "RxJS: observables"
gen nestjs_common "$(v @nestjs/common)" "$backend" @nestjs/common --depends "$TS,rxjs@$(v rxjs)" --description "NestJS common: decorators, pipes, exceptions"
gen nestjs_core "$(v @nestjs/core)" "$backend" @nestjs/core --depends "$TS" --description "NestJS core: NestFactory"
gen sequelize "$(v sequelize)" "$backend" sequelize --depends "$TS" --description "Sequelize: SQL ORM"
gen sequelize_typescript "$(v sequelize-typescript)" "$backend" sequelize-typescript --depends "$TS,sequelize@$(v sequelize)" --description "sequelize-typescript: decorators for Sequelize models"
gen nestjs_sequelize "$(v @nestjs/sequelize)" "$backend" @nestjs/sequelize --depends "$TS" --description "NestJS integration of Sequelize"
gen class_validator "$(v class-validator)" "$backend" class-validator --depends "$TS" --description "class-validator: validation decorators"
gen class_transformer "$(v class-transformer)" "$backend" class-transformer --depends "$TS" --description "class-transformer: plain objects to classes"
gen nestjs_testing "$(v @nestjs/testing)" "$backend" @nestjs/testing --depends "$TS" --description "NestJS testing module"
gen dragonfly-nest "$(v dragonfly-nest)" "$backend" dragonfly-nest dragonfly-nest/ai dragonfly-nest/events dragonfly-nest/testing --depends "$TS" --description "Dragonfly for NestJS: the runtime of the nestjs dialect"
