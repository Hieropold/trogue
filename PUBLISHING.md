# Publishing Trogue

This document describes the process for publishing new versions of `trogue` to the Launchpad PPA and crates.io.

## Prerequisites

Before publishing, ensure that:
1. The version number in `Cargo.toml` has been incremented.
2. The `CHANGELOG.md` (if applicable) or `debian/changelog` is updated.
3. All tests pass: `cargo test`.
4. The code is formatted: `cargo fmt`.
5. Linting passes: `cargo clippy`.

---

## 1. Publishing to Launchpad PPA (Ubuntu)

The project uses a script to automate the creation of Debian source packages and their upload to Launchpad.

### Setup
Ensure you have a `.env` file in the root directory (copy from `.env.example`) with the following variables:
- `DEBFULLNAME`: Your full name (as registered in your GPG key).
- `DEBEMAIL`: Your email address.
- `GPG_KEY_ID`: Your GPG key fingerprint used for signing the package.

Install required system dependencies:
```bash
sudo apt install devscripts dput debhelper dh-cargo
```

### Execution
Run the publishing script:
```bash
./build-and-upload-to-ppa.sh
```

### What the script does:
1. Cleans the project and creates an upstream tarball.
2. Updates `debian/changelog` for each targeted Ubuntu distribution (e.g., Jammy, Noble).
3. Builds a signed source package using `debuild`.
4. Uploads the source package to the PPA using `dput`.
5. Restores the original `debian/changelog` state.

Launchpad will then build the binary packages natively for each architecture. Monitor the build status at:
[https://launchpad.net/~hieropold/+archive/ubuntu/ppa/+packages](https://launchpad.net/~hieropold/+archive/ubuntu/ppa/+packages)

---

## 2. Publishing to crates.io (Rust)

`trogue` can also be published as a library/binary to the official Rust package registry.

### Setup
1. Log in on [crates.io](https://crates.io/) with github acc.
2. Generate an API token in your [Account Settings](https://crates.io/settings/tokens).
3. Authenticate your local machine:
   ```bash
   cargo login <your-api-token>
   ```

### Metadata Verification
Before publishing, ensure `Cargo.toml` contains the necessary metadata for `crates.io`:
- `description`
- `license` (should be a valid SPDX identifier like `Apache-2.0`)
- `repository` (URL to the GitHub repo)
- `readme` (path to README.md)
- `keywords` and `categories` (to improve discoverability)

### Execution
1. **Dry Run**: Verify that the package is ready without actually uploading it.
   ```bash
   cargo publish --dry-run
   ```
   *Note: This will also check if you have uncommitted changes. It is recommended to publish from a clean git state.*

2. **Publish**:
   ```bash
   cargo publish
   ```

### Troubleshooting
- **Vendored Dependencies**: If you see an error like `error: crates-io is replaced with non-remote-registry source`, you may need to specify the registry:
  ```bash
  cargo publish --registry crates-io
  ```
- **Dirty Working Directory**: `cargo publish` will fail if there are uncommitted changes. Either commit your changes or use `--allow-dirty` (not recommended for official releases).

### Post-Publishing
After a successful publish, it is good practice to tag the release in Git:
```bash
git tag v$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
git push origin --tags
```
