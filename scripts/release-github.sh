#!/usr/bin/env bash
#
# GitHub Release Script for Narsil
# Usage: ./scripts/release-github.sh [OPTIONS]
#
# Builds Linux x64 tarball, AppImage, and Windows x64 zip, then creates
# (or updates) a GitHub release with those assets attached.
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${PROJECT_ROOT}/Cargo.toml"
DIST_DIR="${PROJECT_ROOT}/target/dist"

# Parse arguments
DRAFT_MODE=false
SKIP_BUILD=false
SKIP_WINDOWS=false
SKIP_APPIMAGE=false
RELEASE_NOTES=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --draft)
            DRAFT_MODE=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-windows)
            SKIP_WINDOWS=true
            shift
            ;;
        --skip-appimage)
            SKIP_APPIMAGE=true
            shift
            ;;
        --notes)
            RELEASE_NOTES="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Build assets and create/update a GitHub release."
            echo ""
            echo "Options:"
            echo "  --draft              Create the release as a draft"
            echo "  --skip-build         Skip the cargo build step"
            echo "  --skip-windows       Skip Windows cross-compile"
            echo "  --skip-appimage      Skip AppImage build"
            echo "  --notes TEXT         Release notes text"
            echo "  -h, --help           Show this help"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Helper functions
log_info()    { echo -e "${BLUE}ℹ${NC} $1" >&2; }
log_success() { echo -e "${GREEN}✓${NC} $1" >&2; }
log_warning() { echo -e "${YELLOW}⚠${NC} $1" >&2; }
log_error()   { echo -e "${RED}✗${NC} $1" >&2; }
log_step()    { echo -e "${CYAN}${BOLD}▶ $1${NC}" >&2; }

get_version() {
    grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/'
}

get_package_name() {
    grep '^name = ' "$CARGO_TOML" | head -n1 | sed 's/name = "\(.*\)"/\1/'
}

check_gh_cli() {
    if ! command -v gh &>/dev/null; then
        log_error "GitHub CLI (gh) is not installed. Install it from https://cli.github.com/"
        exit 1
    fi
}

check_gh_auth() {
    if ! gh auth status &>/dev/null; then
        log_error "Not authenticated to GitHub. Run: gh auth login"
        exit 1
    fi
}

check_release_exists() {
    local tag="$1"
    gh release view "$tag" &>/dev/null
}

build_release() {
    cd "$PROJECT_ROOT"
    export RUSTUP_TOOLCHAIN=stable

    log_step "Building Linux release binary (standard)..."
    cargo build --release
    cp "target/release/${PACKAGE_NAME}" "target/release/${PACKAGE_NAME}-standard-built"
    log_success "Linux standard binary built"

    log_step "Building Linux release binary (NVIDIA)..."
    cargo build --release --features nvidia
    cp "target/release/${PACKAGE_NAME}" "target/release/${PACKAGE_NAME}-nvidia-built"
    log_success "Linux NVIDIA binary built"

    # Restore standard as the canonical release binary
    cp "target/release/${PACKAGE_NAME}-standard-built" "target/release/${PACKAGE_NAME}"
}

build_windows() {
    if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
        log_warning "mingw cross-compiler not found (x86_64-w64-mingw32-gcc)"
        log_warning "Install with: sudo apt install gcc-mingw-w64-x86-64"
        log_warning "Skipping Windows build"
        return 0
    fi
    if ! rustup target list --installed | grep -q 'x86_64-pc-windows-gnu'; then
        log_info "Adding Windows target for Rust..."
        rustup target add x86_64-pc-windows-gnu
    fi
    cd "$PROJECT_ROOT"

    # Generate .ico from icon.png (project root) so build.rs can embed it into the exe
    if [ -f "icon.png" ] && [ ! -f "icon.ico" ]; then
        if command -v convert &>/dev/null; then
            log_info "Generating icon.ico from icon.png..."
            convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico
        elif command -v magick &>/dev/null; then
            log_info "Generating icon.ico from icon.png..."
            magick icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico
        else
            log_warning "ImageMagick not found — Windows exe will have no custom icon"
            log_warning "Install with: sudo apt install imagemagick"
        fi
    fi

    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
    local win_dir="target/x86_64-pc-windows-gnu/release"

    log_step "Building Windows release binary (standard)..."
    cargo build --release --target x86_64-pc-windows-gnu
    cp "${win_dir}/${PACKAGE_NAME}.exe" "${win_dir}/${PACKAGE_NAME}-standard-built.exe"
    log_success "Windows standard binary built"

    log_step "Building Windows release binary (NVIDIA)..."
    cargo build --release --target x86_64-pc-windows-gnu --features nvidia
    cp "${win_dir}/${PACKAGE_NAME}.exe" "${win_dir}/${PACKAGE_NAME}-nvidia-built.exe"
    log_success "Windows NVIDIA binary built"

    # Restore standard exe
    cp "${win_dir}/${PACKAGE_NAME}-standard-built.exe" "${win_dir}/${PACKAGE_NAME}.exe"
}

create_tarball() {
    local variant="$1"   # standard or nvidia
    local archive_basename dir_name bin_src

    if [ "$variant" = "nvidia" ]; then
        archive_basename="${PACKAGE_NAME}-nvidia-${VERSION}-x86_64.tar.gz"
        dir_name="${PACKAGE_NAME}-nvidia-${VERSION}"
        bin_src="target/release/${PACKAGE_NAME}-nvidia-built"
    else
        archive_basename="${PACKAGE_NAME}-${VERSION}-x86_64.tar.gz"
        dir_name="${PACKAGE_NAME}-${VERSION}"
        bin_src="target/release/${PACKAGE_NAME}-standard-built"
    fi

    # Fallback to canonical binary if stash is missing
    [ -f "$bin_src" ] || bin_src="target/release/${PACKAGE_NAME}"

    local tarball="${DIST_DIR}/${archive_basename}"
    log_step "Creating Linux tarball: ${archive_basename}"

    local staging
    staging=$(mktemp -d)
    local staging_dir="${staging}/${dir_name}"
    mkdir -p "$staging_dir"

    cp "$bin_src" "$staging_dir/${PACKAGE_NAME}"
    [ -f LICENSE ] && cp LICENSE "$staging_dir/"
    [ -f README.md ] && cp README.md "$staging_dir/"

    tar -czf "$tarball" -C "$staging" "${dir_name}"
    rm -rf "$staging"

    log_success "Created: ${archive_basename} ($(du -sh "$tarball" | cut -f1))"
}

create_windows_zip() {
    local variant="$1"   # standard or nvidia

    if ! command -v zip &>/dev/null; then
        log_warning "zip command not found, skipping Windows zip"
        return 0
    fi

    local win_dir="target/x86_64-pc-windows-gnu/release"
    local zip_basename dir_name exe_src

    if [ "$variant" = "nvidia" ]; then
        zip_basename="${PACKAGE_NAME}-nvidia-${VERSION}-x86_64-windows.zip"
        dir_name="${PACKAGE_NAME}-nvidia-${VERSION}"
        exe_src="${win_dir}/${PACKAGE_NAME}-nvidia-built.exe"
    else
        zip_basename="${PACKAGE_NAME}-${VERSION}-x86_64-windows.zip"
        dir_name="${PACKAGE_NAME}-${VERSION}"
        exe_src="${win_dir}/${PACKAGE_NAME}-standard-built.exe"
    fi

    # Fallback if stash is missing
    [ -f "$exe_src" ] || exe_src="${win_dir}/${PACKAGE_NAME}.exe"

    if [ ! -f "$exe_src" ]; then
        log_warning "Windows ${variant} binary not found, skipping zip"
        return 0
    fi

    local zipfile="${DIST_DIR}/${zip_basename}"
    log_step "Creating Windows zip: ${zip_basename}"

    local staging
    staging=$(mktemp -d)
    local staging_dir="${staging}/${dir_name}"
    mkdir -p "$staging_dir"

    cp "$exe_src" "$staging_dir/${PACKAGE_NAME}.exe"
    [ -f LICENSE ] && cp LICENSE "$staging_dir/"
    [ -f README.md ] && cp README.md "$staging_dir/"

    (cd "$staging" && zip -r "$zipfile" "${dir_name}")
    rm -rf "$staging"

    log_success "Created: ${zip_basename} ($(du -sh "$zipfile" | cut -f1))"
}

build_appimage() {
    local std_bin="target/release/${PACKAGE_NAME}-standard-built"
    local nv_bin="target/release/${PACKAGE_NAME}-nvidia-built"

    log_step "Building AppImage (standard)..."
    [ -f "$std_bin" ] && cp "$std_bin" "target/release/${PACKAGE_NAME}"
    if "${PROJECT_ROOT}/scripts/build-appimage.sh" build --variant standard --skip-build; then
        log_success "AppImage (standard) built"
    else
        log_warning "AppImage (standard) build failed — asset will be skipped"
    fi

    log_step "Building AppImage (NVIDIA)..."
    [ -f "$nv_bin" ] && cp "$nv_bin" "target/release/${PACKAGE_NAME}"
    if "${PROJECT_ROOT}/scripts/build-appimage.sh" build --variant nvidia --skip-build; then
        log_success "AppImage (NVIDIA) built"
    else
        log_warning "AppImage (NVIDIA) build failed — asset will be skipped"
    fi

    # Restore standard binary
    [ -f "$std_bin" ] && cp "$std_bin" "target/release/${PACKAGE_NAME}"
}

create_github_release() {
    local tag="v$VERSION"
    local title="⚔️ Narsil v${VERSION}"

    log_step "Creating GitHub release: $tag"

    local -a assets
    assets=()

    # Collect all assets: standard and NVIDIA variants
    local f
    for f in \
        "${DIST_DIR}/${PACKAGE_NAME}-${VERSION}-x86_64.tar.gz" \
        "${DIST_DIR}/${PACKAGE_NAME}-nvidia-${VERSION}-x86_64.tar.gz" \
        "${DIST_DIR}/${PACKAGE_NAME}-${VERSION}-x86_64.AppImage" \
        "${DIST_DIR}/${PACKAGE_NAME}-nvidia-${VERSION}-x86_64.AppImage" \
        "${DIST_DIR}/${PACKAGE_NAME}-${VERSION}-x86_64-windows.zip" \
        "${DIST_DIR}/${PACKAGE_NAME}-nvidia-${VERSION}-x86_64-windows.zip"; do
        if [ -f "$f" ]; then
            assets+=("$f")
            log_info "  + $(basename "$f")"
        fi
    done

    if [ "${#assets[@]}" -eq 0 ]; then
        log_warning "No release assets found in $DIST_DIR"
    fi

    local -a gh_args
    gh_args=(release create "$tag" --title "$title")
    [ "$DRAFT_MODE" = true ] && gh_args+=(--draft)
    [ -n "$RELEASE_NOTES" ] && gh_args+=(--notes "$RELEASE_NOTES") || gh_args+=(--generate-notes)
    gh_args+=(--repo "Pommersche92/narsil")

    if check_release_exists "$tag"; then
        log_warning "Release $tag already exists — deleting and recreating"
        gh release delete "$tag" --yes --repo "Pommersche92/narsil" || true
    fi

    gh "${gh_args[@]}" "${assets[@]}"

    log_success "GitHub release created: https://github.com/Pommersche92/narsil/releases/tag/$tag"
}

main() {
    cd "$PROJECT_ROOT"

    check_gh_cli
    check_gh_auth

    VERSION=$(get_version)
    PACKAGE_NAME=$(get_package_name)

    log_info "Package: $PACKAGE_NAME v$VERSION"
    echo ""

    mkdir -p "$DIST_DIR"

    if [ "$SKIP_BUILD" = false ]; then
        build_release
        echo ""
    fi

    if [ "$SKIP_WINDOWS" = false ]; then
        build_windows
        echo ""
    fi

    if [ "$SKIP_APPIMAGE" = false ]; then
        build_appimage
        echo ""
    fi

    create_tarball standard
    create_tarball nvidia
    create_windows_zip standard || true
    create_windows_zip nvidia || true
    echo ""

    create_github_release

    echo ""
    echo -e "${GREEN}${BOLD}✓ Release assets in: target/dist/${NC}"
    ls -lh "$DIST_DIR" 2>/dev/null || true
}

main
