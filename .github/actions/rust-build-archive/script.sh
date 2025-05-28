#!/bin/bash
set -e

read INPUT_BIN
read INPUT_PROFILE
read INPUT_FEATURES
read INPUT_ARCHIVE
read INPUT_DEB

echo "🔧 bin: $INPUT_BIN"
echo "🔧 profile: $INPUT_PROFILE"
echo "🔧 features: $INPUT_FEATURES"
echo "📁 archive: $INPUT_ARCHIVE"
echo "📦 deb: $INPUT_DEB"

if [[ "$INPUT_PROFILE" != "release" && "$INPUT_PROFILE" != "dev" ]]; then
  echo "❌ Invalid profile: $INPUT_PROFILE. Must be 'dev' or 'release'."
  exit 1
fi

if [[ "$INPUT_PROFILE" == "dev" ]]; then
  BUILD_PATH="target/debug"
else
  BUILD_PATH="target/release"
fi

if [[ "$INPUT_FEATURES" == "" ]]; then
  DEB_CMD="cargo deb --output $INPUT_DEB.deb"
else
  DEB_CMD="cargo deb --features $INPUT_FEATURES --output $INPUT_DEB.deb"
fi

echo "🚧 Building project..."
cargo build --profile "$INPUT_PROFILE"

if [[ -n "$INPUT_DEB" ]]; then
  echo "🛠️ Building DEB package..."
  $DEB_CMD
  echo "📦 DEB archive created: $INPUT_DEB.deb"
fi

mv "$BUILD_PATH/paddler" "$INPUT_BIN"

if [[ -n "$INPUT_ARCHIVE" ]]; then
  tar -czf "$INPUT_BIN.tar.gz" "$INPUT_BIN"
  echo "📦 Binary archive created: $INPUT_ARCHIVE.tar.gz"
fi

echo "✅ Done."
