# Monolai - Modern Local AI Platform
# Task runner configuration (just)

# Display available tasks and usage
default:
    @just --list

# ==============================================================================
# Development
# ==============================================================================

# Start frontend and backend concurrently in development mode
dev:
    #!/usr/bin/env bash
    mkdir -p frontend/build
    trap "kill 0" SIGINT SIGTERM EXIT
    npm --prefix frontend run dev &
    cargo run --manifest-path backend/Cargo.toml &
    wait

# Start frontend (exposed to local network via --host) and backend concurrently
dev-host:
    #!/usr/bin/env bash
    mkdir -p frontend/build
    trap "kill 0" SIGINT SIGTERM EXIT
    npm --prefix frontend run dev -- --host &
    cargo run --manifest-path backend/Cargo.toml &
    wait

# Start frontend development server only (Vite)
dev-frontend:
    npm --prefix frontend run dev

# Start backend cargo server only (Rust Axum)
dev-backend: build-frontend
    cargo run --manifest-path backend/Cargo.toml

# Start native desktop GUI application in development (egui)
dev-desktop:
    cargo build --manifest-path backend/Cargo.toml
    cargo run --manifest-path desktop/Cargo.toml

# ==============================================================================
# Quality, Verification & Testing
# ==============================================================================

# Run type checking (SvelteKit) and compiler checks for backend and desktop
check:
    npm --prefix frontend run check
    cargo check --manifest-path backend/Cargo.toml
    cargo check --manifest-path desktop/Cargo.toml

# Run backend unit and integration tests
test:
    cargo test --manifest-path backend/Cargo.toml

# Validate code formatting and lint rules
lint:
    npm --prefix frontend run lint

# Check for missing translation keys in en.json
i18n-check:
    node scripts/check-i18n.mjs

# Auto-format code across the monorepo (Prettier and rustfmt)
format:
    npm --prefix frontend run format
    cargo fmt --manifest-path backend/Cargo.toml
    cargo fmt --manifest-path desktop/Cargo.toml

# ==============================================================================
# Build & Distribution
# ==============================================================================

# Build frontend into static web assets
build-frontend:
    npm --prefix frontend run build

# Build frontend and release server binary into dist/linux/
build-backend: build-frontend
    cargo build --manifest-path backend/Cargo.toml --release
    @mkdir -p dist/linux
    @cp -f backend/target/release/monolai dist/linux/

# Build native desktop GUI release binary into dist/linux/ (requires backend)
build-desktop: build-backend
    cargo build --manifest-path desktop/Cargo.toml --release
    @mkdir -p dist/linux
    @cp -f desktop/target/release/monolai-gui dist/linux/

# Build complete Linux release bundle (server + desktop GUI)
build-linux: build-backend build-desktop
    @echo "=========================================="
    @echo "Linux build complete! Output in dist/linux/"
    @echo " - dist/linux/monolai"
    @echo " - dist/linux/monolai-gui"
    @echo "=========================================="

# Cross-compile Windows binaries into dist/windows/ (x86_64-pc-windows-gnu)
build-windows: build-frontend
    cargo build --manifest-path backend/Cargo.toml --release --target x86_64-pc-windows-gnu
    cargo build --manifest-path desktop/Cargo.toml --release --target x86_64-pc-windows-gnu
    @mkdir -p dist/windows
    @cp -f backend/target/x86_64-pc-windows-gnu/release/monolai.exe dist/windows/
    @cp -f desktop/target/x86_64-pc-windows-gnu/release/monolai-gui.exe dist/windows/
    @echo "=========================================="
    @echo "Windows build complete! Output in dist/windows/"
    @echo " - dist/windows/monolai.exe"
    @echo " - dist/windows/monolai-gui.exe"
    @echo "=========================================="

# Build release binaries and macOS application bundle (requires macOS host)
build-macos:
    #!/usr/bin/env bash
    if [ "$(uname -s)" = "Darwin" ]; then \
        npm --prefix frontend run build && \
        cargo build --manifest-path backend/Cargo.toml --release && \
        cargo build --manifest-path desktop/Cargo.toml --release && \
        mkdir -p dist/macos/Monolai.app/Contents/MacOS dist/macos/Monolai.app/Contents/Resources && \
        cp -f backend/target/release/monolai dist/macos/ && \
        cp -f desktop/target/release/monolai-gui dist/macos/ && \
        cp -f backend/target/release/monolai dist/macos/Monolai.app/Contents/MacOS/ && \
        cp -f desktop/target/release/monolai-gui dist/macos/Monolai.app/Contents/MacOS/ && \
        cp -f packaging/macos/Info.plist dist/macos/Monolai.app/Contents/ && \
        (cp -f packaging/macos/assets/monolai.icns dist/macos/Monolai.app/Contents/Resources/ 2>/dev/null || true) && \
        echo "==========================================" && \
        echo "macOS build complete! Output in dist/macos/" && \
        echo " - dist/macos/monolai" && \
        echo " - dist/macos/monolai-gui" && \
        echo " - dist/macos/Monolai.app" && \
        echo "=========================================="; \
    else \
        echo "Notice: Compiling for macOS requires running on macOS or configuring an osxcross SDK." >&2; \
        exit 1; \
    fi

# Alias for build-macos
build-mac: build-macos

# Default release build for current host platform (Linux)
build: build-linux

# Build all supported multi-platform release binaries
build-all: build-linux build-windows

# Build local Docker image
build-docker:
    docker build -t monolai:latest -f docker/Dockerfile .

# ==============================================================================
# Maintenance & Utilities
# ==============================================================================

# Install frontend dependencies and verify Rust workspaces
install:
    mkdir -p frontend/build
    npm --prefix frontend install
    cargo check --manifest-path backend/Cargo.toml
    cargo check --manifest-path desktop/Cargo.toml

# Generate application icon assets (target: all, linux, macos, or windows)
icons target="all":
    ./packaging/scripts/generate-icons.sh --{{target}}

# Remove build artifacts and temporary compilation directories
clean:
    rm -rf frontend/build frontend/.svelte-kit dist dist-windows
    cargo clean --manifest-path backend/Cargo.toml
    cargo clean --manifest-path desktop/Cargo.toml

# Display current version or synchronize across manifests and packaging scripts (e.g., just version or just version 0.2.0)
version new_version="":
    #!/usr/bin/env bash
    NEW_V="{{new_version}}"
    NEW_V="${NEW_V#v}"
    if [ -z "$NEW_V" ]; then
        sed -n -E 's/^version = "([^"]+)"/\1/p' backend/Cargo.toml | head -n 1
        exit 0
    fi
    mkdir -p frontend/build
    sed -i -E "0,/^version = \"[^\"]+\"/s//version = \"$NEW_V\"/" backend/Cargo.toml
    sed -i -E "0,/^version = \"[^\"]+\"/s//version = \"$NEW_V\"/" desktop/Cargo.toml
    sed -i -E "0,/\"version\": \"[^\"]+\"/s//\"version\": \"$NEW_V\"/" frontend/package.json
    echo "export const APP_VERSION = '$NEW_V';" > frontend/src/lib/version.ts
    if [ -f packaging/windows/monolai.nsi ]; then
        sed -i -E 's/(!define VERSION ")[^"]*(")/\1'"$NEW_V"'\2/' packaging/windows/monolai.nsi
    fi
    (cd backend && cargo check --quiet)
    (cd desktop && cargo check --quiet)
    printf "Version synchronized to %s\n" "$NEW_V"
