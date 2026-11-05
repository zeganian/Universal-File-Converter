# Continuous Integration

## Universal File Converter

This document describes what the pipeline of this repository verifies,
which versions it uses, and how to reproduce every job locally.

The rules that apply to the pipeline:

- every dependency is pinned or has a tested compatibility range;
- no package repository that cannot be verified is used;
- the integration tests are never skipped to make the pipeline green;
- the business logic is never changed to work around a pipeline problem;
- the lockfiles stay committed and unchanged.

## Jobs

| Job            | Platform              | What it verifies                                                         |
| -------------- | --------------------- | ------------------------------------------------------------------------ |
| `frontend`     | ubuntu-latest         | Formatting, linting, type check, unit tests and the production build.    |
| `rust-core`    | ubuntu-latest         | The domain crate: formatting, linting, tests and the generated bindings. |
| `rust-engines` | ubuntu-latest         | The engine crate with a real ImageMagick 7 and real conversions.         |
| `rust-desktop` | ubuntu-latest         | The Tauri crate with the system dependencies of the platform.            |
| `build`        | Linux, Windows, macOS | The whole workspace compiles on every supported platform.                |

The `build` job runs with `fail-fast: false`, so a failure on one
platform does not hide the result of the other platforms.

## Pinned versions

| Dependency  | Version  | Where it is pinned                                            |
| ----------- | -------- | ------------------------------------------------------------- |
| Node.js     | 22       | `.github/workflows/ci.yml`                                    |
| pnpm        | 12.16.1  | `package.json` (`packageManager`)                             |
| Rust        | stable   | `dtolnay/rust-toolchain@stable`                               |
| ImageMagick | 7.1.2-35 | `scripts/install-imagemagick.sh` and the workflow environment |

The Rust dependencies are pinned by `Cargo.lock`, the frontend
dependencies by `pnpm-lock.yaml`. Both lockfiles are committed, and
every job uses `--locked` or `--frozen-lockfile` so that a job fails
instead of silently updating a version.

## ImageMagick 7

Ubuntu 24.04 only ships ImageMagick 6, and the community repositories
that carry ImageMagick 7 are not published for every release. The
pipeline therefore builds the engine itself:

1. `scripts/install-imagemagick.sh` downloads the official source
   release `ImageMagick-7.1.2-35.tar.xz` from the release of the
   ImageMagick project on GitHub;
2. the SHA-256 of the archive is verified against the value that is
   pinned in the script, and the download is rejected when it differs;
3. the engine is compiled with `--with-png=yes --with-jpeg=yes`, so the
   two formats of the implemented pair are always present;
4. the installation is verified: the version has to be ImageMagick 7 and
   has to be the pinned version.

The built prefix is cached, so only the first run of a version pays for
the compilation.

The `rust-engines` job fails when the engine is missing, when it is not
ImageMagick 7, when it is not the pinned version, or when the PNG or
the JPEG delegate is missing. The integration test of the PNG to JPEG
pair skips itself when no engine is found, so the job additionally
checks that no test was skipped. A conversion can therefore never be
reported as tested without a real conversion.

To reproduce the installation locally:

```bash
sudo -E ./scripts/install-imagemagick.sh            # into /opt/imagemagick
IMAGEMAGICK_PREFIX="$HOME/imagemagick" ./scripts/install-imagemagick.sh
```

The script needs a C compiler, `make`, `pkg-config`, `gettext`,
`libpng` and `libjpeg` headers. On Debian and Ubuntu:

```bash
sudo apt-get install build-essential pkg-config gettext libpng-dev libjpeg-turbo8-dev zlib1g-dev
```

## System dependencies per operating system

### Linux (ubuntu-latest, Debian and Ubuntu)

The Tauri 2 dependencies of the desktop crate:

```text
libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
librsvg2-dev libxdo-dev libssl-dev patchelf build-essential file
pkg-config wget curl
```

`libayatana-appindicator3-dev` replaces the original `libappindicator3-dev`.
The two packages conflict with each other, so exactly one of them may
be installed. The Ayatana package is the one that Tauri 2 documents for
Ubuntu 24.04.

### macOS (macos-latest)

The build needs the command line tools of Xcode, which the runner image
already contains. No Homebrew package is installed: the pipeline does
not depend on a package manager for the build.

### Windows (windows-latest)

The build needs the Visual Studio build tools, which the runner image
already contains. No engine is installed on this platform yet, so the
engine tests are skipped there by design.

## Reproducing the jobs locally

```bash
pnpm install --frozen-lockfile
pnpm check            # formatting, linting, types, bindings, tests

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
node scripts/generate-fixtures.mjs
```

## Not verified by the pipeline yet

- The release gates of `TESTING.md` section 5 for the other platforms:
  the PNG to JPEG pair is only tested with a real conversion on Linux.
- The installer of the desktop application, the code signing and the
  notarisation of the macOS build.
- The engines on Windows and macOS: no adapter is available there yet.
