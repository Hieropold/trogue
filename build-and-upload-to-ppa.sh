#!/bin/bash

# Exit on error
set -e

# Clean the project
cargo clean

# Create the upstream tarball
VERSION=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Create the upstream tarball
tar --exclude='./.git' --exclude='./debian' -czf ../trogue_${VERSION}.orig.tar.gz .

# Build the source package
debuild -S

# Sign the source package with hieropold's GPG key
debsign -k 995BE09B4F8CC7B8236CE3B35DBE9408AE12691B ../trogue_${VERSION}-1_source.changes

# Upload to Launchpad PPA
dput ppa:hieropold/ppa ../trogue_${VERSION}-1_source.changes

echo "Script created successfully."
echo "Please edit the script to add your GPG key ID, Launchpad ID, and PPA name."
echo "Then you can run it to build and upload the package."
