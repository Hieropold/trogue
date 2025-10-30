#!/bin/bash

# Exit on error
set -e

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

# Update debian/changelog to match the version
dch -v ${VERSION}-1 "New upstream release ${VERSION}" || true
dch -r "" || true

# Build and sign the source package with hieropold's GPG key
debuild -S -sa -k995BE09B4F8CC7B8236CE3B35DBE9408AE12691B

# Upload to Launchpad PPA
dput ppa:hieropold/ppa ../trogue_${VERSION}-1_source.changes