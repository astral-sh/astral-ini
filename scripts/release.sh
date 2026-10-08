#!/usr/bin/env bash
# Prepare for a release.
#
# All additional options are passed to `rooster`.
set -euo pipefail

cd "$(dirname "$0")/.."
if [[ -n "$(git status --porcelain)" ]]; then
    echo "Release preparation requires a clean checkout" >&2
    exit 1
fi

# Rooster expects version files to start at 0.0.0 when there is no previous
# tag. Our first release starts at the version already in the manifests.
if ! git tag --list | grep -Eq '^v?[0-9]+\.[0-9]+\.[0-9]+$'; then
    if [[ $# == 0 ]]; then
        set -- --version "$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["workspace"]["package"]["version"])')"
    fi
    uv run --locked --python 3.12 --only-group release rooster release --no-update-version-files "$@"
else
    uv run --locked --python 3.12 --only-group release rooster release "$@"
fi

# Keep the workspace version in sync with the generated changelog.
version="$(python3 scripts/update-release-version.py)"

echo "Updating lockfiles..."
cargo update -p astral-ini
cargo update --manifest-path fuzz/Cargo.toml -p astral-ini

echo "Creating release branch..."
git checkout -b "release/$version"
# The first release creates the changelog rather than updating an existing file.
git add CHANGELOG.md
git commit -am "Bump version to $version"
