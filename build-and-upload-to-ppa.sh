#!/bin/bash

# Builds signed Debian source packages for each target Ubuntu series and
# uploads them to the Launchpad PPA. Launchpad then builds the binary
# package natively inside a per-series chroot, ensuring correct library
# dependencies for each Ubuntu version.

# Exit on error
set -e

# ---------------------------------------------------------------------------
# Load configuration
# ---------------------------------------------------------------------------
# Maintainer name, email, and GPG key are read from .env so they are never
# committed to the repository. Copy .env.example to .env and fill in values.
if [[ ! -f .env ]]; then
    echo "Error: .env file not found. Copy .env.example to .env and fill in your values."
    exit 1
fi
# shellcheck source=.env.example
source .env

for var in DEBFULLNAME DEBEMAIL GPG_KEY_ID; do
    if [[ -z "${!var}" ]]; then
        echo "Error: '$var' is not set in .env."
        exit 1
    fi
done

export DEBFULLNAME
export DEBEMAIL

# ---------------------------------------------------------------------------
# Prerequisites
# ---------------------------------------------------------------------------
for cmd in dch debuild dput gpg cargo; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "Error: '$cmd' is not installed. Please install it and try again."
        exit 1
    fi
done

# ---------------------------------------------------------------------------
# Changelog guard
# ---------------------------------------------------------------------------
# Store original changelog in a variable to restore later.
# This ensures that our temporary modifications for the build don't persist
# and avoids issues with dh_clean deleting backup files.
ORIG_CHANGELOG=$(cat debian/changelog)
trap 'printf "%s" "$ORIG_CHANGELOG" > debian/changelog' EXIT

# ---------------------------------------------------------------------------
# Prepare sources
# ---------------------------------------------------------------------------
# Clean the project
cargo clean

# Get version from Cargo.toml
VERSION=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "Packaging version $VERSION"

# Create the upstream tarball (vendor/ is included so Launchpad can build
# offline; .git and debian/ are excluded as they are not upstream sources).
tar --exclude='./.git' --exclude='./debian' -czf "../trogue_${VERSION}.orig.tar.gz" .

# ---------------------------------------------------------------------------
# Per-series build and upload
# ---------------------------------------------------------------------------
for DISTRO in jammy noble; do
    echo "----------------------------------------------------------------"
    echo "Building for distribution: $DISTRO"
    echo "----------------------------------------------------------------"

    # Reset changelog to the unmodified state before each iteration so that
    # dch always inserts a single fresh entry at the top.
    printf "%s" "$ORIG_CHANGELOG" > debian/changelog

    # Remove stale Debian build-artifact files that could confuse debuild.
    # debian/files is auto-generated during a build; a leftover copy from a
    # previous run can cause dpkg-genchanges to include incorrect entries.
    rm -f debian/files

    # Remove leftover source-package files from a previous run of this loop
    # iteration so that dput does not re-upload stale artefacts.
    rm -f "../trogue_${VERSION}-1~${DISTRO}1"_*.changes \
          "../trogue_${VERSION}-1~${DISTRO}1"*.dsc \
          "../trogue_${VERSION}-1~${DISTRO}1"*.buildinfo \
          2>/dev/null || true

    # Append ~distro1 to the version to ensure each series gets a unique,
    # correctly-targeted package version (e.g. 0.2.2-1~jammy1).
    dch -D "$DISTRO" -v "${VERSION}-1~${DISTRO}1" "New upstream release ${VERSION}"
    dch -r ""

    # Build the signed source-only package using hieropold's GPG key.
    # -S  : source package only (Launchpad handles binary compilation)
    # -sa : always include the orig tarball
    debuild -S -sa -d -k"$GPG_KEY_ID"

    # Upload to Launchpad PPA
    dput ppa:hieropold/ppa "../trogue_${VERSION}-1~${DISTRO}1_source.changes"

    echo "Uploaded $DISTRO package successfully."
done

echo "----------------------------------------------------------------"
echo "All series uploaded. Monitor build status at:"
echo "  https://launchpad.net/~hieropold/+archive/ubuntu/ppa/+packages"
echo "----------------------------------------------------------------"
