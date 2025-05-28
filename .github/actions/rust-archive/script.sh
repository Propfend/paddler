#!/bin/bash
set -e

read INPUT_ARCHIVE
read INPUT_DEB
read INPUT_PATH

echo "📁 archive: $INPUT_ARCHIVE"
echo "📦 deb: $INPUT_DEB"
echo "📦 project: $INPUT_PATH"

if [[ -n "$INPUT_DEB" ]]; then
  echo "🛠️ Building DEB package..."
  cargo deb --no-build --output $INPUT_DEB.deb

  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'deb=%s\n' "${INPUT_DEB}" >>"${GITHUB_OUTPUT}"
  else
    echo "GITHUB_OUTPUT is not set; skip setting the 'archive' output"
    echo "📦 DEB archive created: $INPUT_DEB.deb"
  fi  
fi

if [[ -n "$INPUT_ARCHIVE" ]]; then
  tar -czf "$INPUT_ARCHIVE.tar.gz" "$INPUT_PATH"
  
  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'archive=%s\n' "${INPUT_ARCHIVE}" >>"${GITHUB_OUTPUT}"
  else
    echo "GITHUB_OUTPUT is not set; skip setting the 'archive' output"
    echo "📦 Compressed archive created: $INPUT_ARCHIVE.tar.gz"
  fi
fi

echo "✅ Done."
