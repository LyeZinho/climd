#!/bin/sh
# climd installer script
# Usage: curl -sL https://raw.githubusercontent.com/LyeZinho/climd/main/install.sh | sh

set -e

REPO="LyeZinho/climd"
INSTALL_DIR="${HOME}/.local/bin"

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*)  echo "macos" ;;
        *)        echo "unknown" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64)   echo "x86_64" ;;
        aarch64)  echo "aarch64" ;;
        arm64)    echo "aarch64" ;;
        *)        echo "unknown" ;;
    esac
}

download_release() {
    os="$1"
    arch="$2"
    version="$3"

    case "$os" in
        linux)
            case "$arch" in
                x86_64)  suffix="x86_64-unknown-linux-gnu.tar.gz" ;;
                aarch64) suffix="aarch64-unknown-linux-gnu.tar.gz" ;;
                *)       echo "Unsupported architecture: $arch"; exit 1 ;;
            esac
            ;;
        macos)
            case "$arch" in
                x86_64)  suffix="x86_64-apple-darwin.tar.gz" ;;
                aarch64) suffix="aarch64-apple-darwin.tar.gz" ;;
                *)       echo "Unsupported architecture: $arch"; exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $os"
            exit 1
            ;;
    esac

    url="https://github.com/${REPO}/releases/download/${version}/climd-${suffix}"
    echo "Downloading from: $url"

    curl -sL "$url" | tar xz -C /tmp

    mkdir -p "$INSTALL_DIR"
    mv /tmp/climd "$INSTALL_DIR/climd"
    chmod +x "$INSTALL_DIR/climd"

    echo "Installed climd to $INSTALL_DIR/climd"

    if [ -d "$INSTALL_DIR" ] && [ ":$PATH:" = *":$INSTALL_DIR:"* ]; then
        echo "Make sure $INSTALL_DIR is in your PATH"
    fi
}

get_latest_version() {
    curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4
}

main() {
    os=$(detect_os)
    arch=$(detect_arch)

    if [ "$os" = "unknown" ]; then
        echo "Unsupported operating system"
        exit 1
    fi

    version="${1:-$(get_latest_version)}"

    echo "Installing climd ${version} for ${os}-${arch}..."

    download_release "$os" "$arch" "$version"

    echo "Done! Run 'climd' to start."
}

main "$@"
