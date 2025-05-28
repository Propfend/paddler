#!/bin/bash
set -e

echo "🔧 bin: $INPUT_BIN"
echo "🔧 profile: $INPUT_PROFILE"
echo "🔧 features: $INPUT_FEATURES"

if [[ $INPUT_PROFILE != "release" && $INPUT_PROFILE != "dev" ]]; then
  echo "❌ Invalid profile: $INPUT_PROFILE. Must be 'dev' or 'release'."
  exit 1
fi

if [[ $INPUT_PROFILE == "dev" ]]; then
  BUILD_PATH="target/debug"
else
  BUILD_PATH="target/release"
fi

if [[ $INPUT_FEATURES == "" ]]; then
  BUILD_CMD="cargo build --profile $INPUT_PROFILE"
else
  BUILD_CMD="cargo build --profile $INPUT_PROFILE --features $INPUT_FEATURES"
fi

echo "🚧 Building project..."
$BUILD_CMD

mv "$BUILD_PATH/paddler" "$INPUT_BIN"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  printf 'bin=%s\n' "${INPUT_BIN}" >>"${GITHUB_OUTPUT}"
else
  echo "GITHUB_OUTPUT is not set; skip setting the 'archive' output"
  echo "📦 Binary archive created: $INPUT_BIN"
fi

echo "✅ Done."
