#!/bin/bash
set -e

read INPUT_ARCHIVE
read INPUT_DEB
read INPUT_PROJECT
read INPUT_PROFILE

echo "📁 archive: $INPUT_ARCHIVE"
echo "📦 deb: $INPUT_DEB"
echo "📦 project: $INPUT_PROJECT"
echo "📦 profile: $INPUT_PROFILE"

if [[ "$INPUT_PROFILE" == "dev" ]]; then
  BUILD_PATH="target/debug/$INPUT_PROJECT"
else
  BUILD_PATH="target/release/$INPUT_PROJECT"
fi

if [[ "$INPUT_FEATURES" == "" ]]; then
  DEB_CMD="cargo deb --output $INPUT_DEB.deb"
else
  DEB_CMD="cargo deb --features $INPUT_FEATURES --output $INPUT_DEB.deb"
fi

if [[ -n "$INPUT_DEB" ]]; then
  echo "🛠️ Building DEB package..."
  $DEB_CMD
  echo "📦 DEB archive created: $INPUT_DEB.deb"
fi

if [[ -n "$INPUT_ARCHIVE" ]]; then
  tar -czf "$INPUT_ARCHIVE.tar.gz" "$BUILD_PATH"
  echo "📦 Binary archive created: $INPUT_ARCHIVE.tar.gz"
fi

echo "archive=$INPUT_ARCHIVE.tar.gz" >> "$GITHUB_OUTPUT"
echo "deb=$INPUT_DEB.deb" >> "$GITHUB_OUTPUT"

echo "✅ Done."
