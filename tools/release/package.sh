#!/bin/sh
# The release archive of NuclearLang for one target, what the installers download (install.sh, install.ps1):
#   dist/nuclearlang-<version>-<os>-<arch>.tar.gz   (.zip for Windows)   and its .sha256
# holding nuclearlang/ = the installation's layout (tools/release/stage.sh) and its VERSION.
#
#   tools/release/package.sh                     for this machine
#   tools/release/package.sh <rust target>       x86_64-unknown-linux-musl, aarch64-apple-darwin, x86_64-pc-windows-msvc...
#
# The JavaScript runtimes are built before (npm run build in each packages/<name> with a package.json); the version is
# the one of the workspace (Cargo.toml [workspace.package]).
set -eu
REPO=$(cd "$(dirname "$0")/../.." && pwd)
TARGET=${1:-}
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$REPO/Cargo.toml" | head -n 1)

# os and arch of the archive's name, from the target (or this machine)
triple=${TARGET:-$(rustc -vV | sed -n 's/^host: //p')}
case "$triple" in
  *linux*) os=linux ;;
  *apple*) os=macos ;;
  *windows*) os=windows ;;
  *) echo "unknown target: $triple" >&2; exit 1 ;;
esac
case "$triple" in
  x86_64*) arch=x86_64 ;;
  aarch64*|arm64*) arch=aarch64 ;;
  *) echo "unknown architecture: $triple" >&2; exit 1 ;;
esac
EXE=""
[ "$os" = windows ] && EXE=".exe"

# the three binaries, each from its workspace
for ws in "$REPO" "$REPO/dragonfly"; do
  pkg=""
  [ "$ws" = "$REPO" ] && pkg="-p nuclear-cli -p nuclear-lsp"
  # shellcheck disable=SC2086
  cargo build --release --quiet --manifest-path "$ws/Cargo.toml" $pkg ${TARGET:+--target "$TARGET"}
done
out=release
[ -n "$TARGET" ] && out="$TARGET/release"

name="nuclearlang-$VERSION-$os-$arch"
dist="$REPO/dist"
stage="$dist/$name/nuclearlang"
rm -rf "$dist/$name"
"$REPO/tools/release/stage.sh" "$stage" \
  "$REPO/target/$out/nuclear$EXE" "$REPO/dragonfly/target/$out/dragonfly$EXE" "$REPO/target/$out/nuclear-lsp$EXE"
echo "$VERSION" > "$stage/VERSION"

cd "$dist/$name"
if [ "$os" = windows ]; then
  archive="$name.zip"
  rm -f "../$archive"
  if command -v zip >/dev/null 2>&1; then zip -qr "../$archive" nuclearlang
  else powershell -NoProfile -Command "Compress-Archive -Path nuclearlang -DestinationPath ../$archive"; fi
else
  archive="$name.tar.gz"
  tar -czf "../$archive" nuclearlang
fi
cd "$dist"
rm -rf "$name"
if command -v sha256sum >/dev/null 2>&1; then sha256sum "$archive" > "$archive.sha256"
else shasum -a 256 "$archive" > "$archive.sha256"; fi
# `latest`: the version a mirror serves (NUCLEAR_DOWNLOAD of the installers)
echo "$VERSION" > "$dist/latest"
echo "$dist/$archive"
