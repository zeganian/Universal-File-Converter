#!/usr/bin/env bash
#
# Installs the ImageMagick 7 that the integration tests use.
#
# The version and the checksum of the official source release are pinned
# in this file. The archive is downloaded from the release of the
# ImageMagick project on GitHub and the checksum is verified before
# anything is built, so that every run installs exactly the same bits.
#
# The script builds ImageMagick from source instead of installing a
# distribution package:
#
# - Ubuntu 24.04 only ships ImageMagick 6, and the community package
#   repositories that carry ImageMagick 7 are not available for every
#   release, which makes the installation depend on a third party;
# - a source build with a pinned checksum is reproducible and can be
#   verified by anybody;
# - the build flags state exactly which formats are supported, so the
#   PNG and JPEG delegates are always present.
#
# The script never replaces an ImageMagick that is already installed: it
# installs into its own prefix.
#
# Usage:
#   ./scripts/install-imagemagick.sh
#   IMAGEMAGICK_PREFIX=/tmp/imagemagick ./scripts/install-imagemagick.sh
#
# Environment variables:
#   IMAGEMAGICK_VERSION   The version to install. Default: 7.1.2-35.
#   IMAGEMAGICK_SHA256    The checksum of the source archive.
#   IMAGEMAGICK_PREFIX    The installation prefix. Default: /opt/imagemagick.
#   IMAGEMAGICK_JOBS      The number of parallel build jobs. Default: nproc.

set -euo pipefail

IMAGEMAGICK_VERSION="${IMAGEMAGICK_VERSION:-7.1.2-35}"
IMAGEMAGICK_SHA256="${IMAGEMAGICK_SHA256:-9554556c8cbf7d347cbd6bedae931ccf323a2bd5e0f08d4834856568ee19f080}"
IMAGEMAGICK_PREFIX="${IMAGEMAGICK_PREFIX:-/opt/imagemagick}"
IMAGEMAGICK_JOBS="${IMAGEMAGICK_JOBS:-$(nproc 2>/dev/null || echo 2)}"

ARCHIVE="ImageMagick-${IMAGEMAGICK_VERSION}.tar.xz"
URL="https://github.com/ImageMagick/ImageMagick/releases/download/${IMAGEMAGICK_VERSION}/${ARCHIVE}"

say() {
  printf '==> %s\n' "$1"
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

require() {
  command -v "$1" >/dev/null 2>&1 || fail "'$1' is required, but it was not found."
}

[ "$(uname -s)" = "Linux" ] || fail "This script installs ImageMagick on Linux only."
[ "$(uname -m)" = "x86_64" ] || fail "This script builds ImageMagick for x86_64 only."

for tool in curl tar make gcc pkg-config sha256sum; do
  require "$tool"
done

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

# The prefix has to be writable, otherwise the installation fails with a
# permission error in the middle of the build.
if [ -e "$IMAGEMAGICK_PREFIX" ]; then
  [ -w "$IMAGEMAGICK_PREFIX" ] ||
    fail "'$IMAGEMAGICK_PREFIX' is not writable. Run the script with sudo, or set IMAGEMAGICK_PREFIX to a writable directory."
else
  parent="$(dirname "$IMAGEMAGICK_PREFIX")"
  while [ ! -d "$parent" ] && [ "$parent" != "/" ]; do
    parent="$(dirname "$parent")"
  done
  [ -w "$parent" ] ||
    fail "'$parent' is not writable. Run the script with sudo, or set IMAGEMAGICK_PREFIX to a writable directory."
fi

say "Downloading ${ARCHIVE}"
curl --fail --silent --show-error --location --retry 3 --output "$WORKDIR/$ARCHIVE" "$URL"

say "Verifying the checksum"
echo "${IMAGEMAGICK_SHA256}  ${WORKDIR}/${ARCHIVE}" | sha256sum --check --status ||
  fail "The checksum of ${ARCHIVE} does not match the pinned value. The download was rejected."

say "Extracting the source"
tar --extract --xz --file "$WORKDIR/$ARCHIVE" --directory "$WORKDIR"

SOURCE_DIR="$WORKDIR/ImageMagick-${IMAGEMAGICK_VERSION}"
[ -d "$SOURCE_DIR" ] || fail "The archive does not contain the expected source directory."

say "Configuring the build"
# Only the PNG and the JPEG delegate are needed for the implemented
# conversion pair. The build stays small and does not depend on the
# optional libraries of the distribution.
(
  cd "$SOURCE_DIR"
  ./configure \
    --prefix="$IMAGEMAGICK_PREFIX" \
    --with-png=yes \
    --with-jpeg=yes \
    --without-x \
    --without-magick-plus-plus \
    --disable-docs \
    --disable-openmp \
    >configure.log 2>&1 || {
      tail -40 configure.log >&2
      exit 1
    }
)

say "Building with ${IMAGEMAGICK_JOBS} jobs"
make -C "$SOURCE_DIR" --jobs="$IMAGEMAGICK_JOBS" >/dev/null
make -C "$SOURCE_DIR" install >/dev/null

MAGICK="$IMAGEMAGICK_PREFIX/bin/magick"
[ -x "$MAGICK" ] || fail "The build did not produce '$MAGICK'."

say "Verifying the installed engine"
VERSION_OUTPUT="$("$MAGICK" -version)"
printf '%s\n' "$VERSION_OUTPUT"

case "$VERSION_OUTPUT" in
  *"ImageMagick 7."*) ;;
  *) fail "The installed engine is not ImageMagick 7." ;;
esac

case "$VERSION_OUTPUT" in
  *"ImageMagick ${IMAGEMAGICK_VERSION}"*) ;;
  *) fail "The installed engine is not version ${IMAGEMAGICK_VERSION}." ;;
esac

say "ImageMagick ${IMAGEMAGICK_VERSION} is installed in ${IMAGEMAGICK_PREFIX}"
