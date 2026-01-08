#!/bin/bash

# Exit on error
set -e

# Store original changelog in a variable to restore later
# This ensures that our temporary modifications for the build don't persist
# and avoids issues with dh_clean deleting backup files.
ORIG_CHANGELOG=$(cat debian/changelog)
trap 'printf "%s" "$ORIG_CHANGELOG" > debian/changelog' EXIT

# Set maintainer information for dch
export DEBFULLNAME="Hieropold"
export DEBEMAIL="hieropold@gmail.com"

# Clean the project
cargo clean

# Get version from Cargo.toml
VERSION=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "Packaging version $VERSION"

# Create the upstream tarball
tar --exclude='./.git' --exclude='./debian' -czf ../trogue_${VERSION}.orig.tar.gz .

# Loop through target distributions
for DISTRO in jammy noble; do
    echo "----------------------------------------------------------------"
    echo "Building for distribution: $DISTRO"
    echo "----------------------------------------------------------------"

    # Reset changelog to original state for this iteration
    printf "%s" "$ORIG_CHANGELOG" > debian/changelog

    # Update debian/changelog for specific distribution
    # Append ~distro1 to version to ensure uniqueness and correct targeting
    dch -D "$DISTRO" -v "${VERSION}-1~${DISTRO}1" "New upstream release ${VERSION} for ${DISTRO}" || true
    dch -r "" || true

    # Build and sign the source package with hieropold's GPG key
    debuild -S -sa -k995BE09B4F8CC7B8236CE3B35DBE9408AE12691B

    # Upload to Launchpad PPA
    dput ppa:hieropold/ppa "../trogue_${VERSION}-1~${DISTRO}1_source.changes"
done