ifeq (version,$(firstword $(MAKECMDGOALS)))
  VERSION_ARG := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(VERSION_ARG):;@:)
endif

.PHONY: help all dev dev-host dev-frontend dev-backend dev-desktop build build-server build-desktop build-linux build-windows build-mac build-all icons install clean version docker-build docker-up docker-down

.DEFAULT_GOAL := help

help:
	@printf "%-18s %s\n" "Target" "Description"
	@printf "%-18s %s\n" "------" "-----------"
	@printf "%-18s %s\n" "help" "Show this help message"
	@printf "%-18s %s\n" "install" "Install frontend and backend dependencies"
	@printf "%-18s %s\n" "dev" "Start frontend and backend in development mode"
	@printf "%-18s %s\n" "dev-host" "Start frontend (--host) and backend in development mode"
	@printf "%-18s %s\n" "dev-frontend" "Start frontend dev server only"
	@printf "%-18s %s\n" "dev-backend" "Start backend cargo server only"
	@printf "%-18s %s\n" "dev-desktop" "Start native Rust desktop GUI app"
	@printf "%-18s %s\n" "build" "Build Linux server and desktop release binaries into dist/linux/"
	@printf "%-18s %s\n" "build-server" "Build frontend and release server binary into dist/linux/"
	@printf "%-18s %s\n" "build-desktop" "Build native desktop GUI binary into dist/linux/"
	@printf "%-18s %s\n" "build-linux" "Build Linux release binaries into dist/linux/"
	@printf "%-18s %s\n" "build-windows" "Cross-compile monolai.exe and monolai-gui.exe into dist/windows/"
	@printf "%-18s %s\n" "build-mac" "Build macOS release binaries into dist/macos/ (when on macOS)"
	@printf "%-18s %s\n" "build-all" "Build all supported platform binaries into dist/<os>/"
	@printf "%-18s %s\n" "icons" "Generate app icons (PNGs, ICO, ICNS) from SVG"
	@printf "%-18s %s\n" "docker-build" "Build Docker image locally"
	@printf "%-18s %s\n" "docker-up" "Start services with docker compose"
	@printf "%-18s %s\n" "docker-down" "Stop services with docker compose"
	@printf "%-18s %s\n" "clean" "Remove build artifacts and dist/ directory"
	@printf "%-18s %s\n" "version" "Synchronize project version (usage: make version [x.y.z])"

all: build

install:
	mkdir -p frontend/build
	npm --prefix frontend install
	cargo check --manifest-path backend/Cargo.toml

dev:
	@mkdir -p frontend/build
	@bash -c '\
		trap "kill 0" SIGINT SIGTERM EXIT; \
		npm --prefix frontend run dev & \
		cargo run --manifest-path backend/Cargo.toml & \
		wait \
	'

dev-host:
	@mkdir -p frontend/build
	@bash -c '\
		trap "kill 0" SIGINT SIGTERM EXIT; \
		npm --prefix frontend run dev -- --host & \
		cargo run --manifest-path backend/Cargo.toml & \
		wait \
	'

dev-frontend:
	npm --prefix frontend run dev

dev-backend:
	mkdir -p frontend/build
	cargo run --manifest-path backend/Cargo.toml

dev-desktop:
	cargo run --manifest-path desktop/Cargo.toml

build-server:
	npm --prefix frontend run build
	cargo build --manifest-path backend/Cargo.toml --release
	@mkdir -p dist/linux
	@cp -f backend/target/release/monolai dist/linux/

build-desktop:
	cargo build --manifest-path desktop/Cargo.toml --release
	@mkdir -p dist/linux
	@cp -f desktop/target/release/monolai-gui dist/linux/

build-linux: build-server build-desktop
	@echo "=========================================="
	@echo "Linux build complete! Output in dist/linux/"
	@echo " - dist/linux/monolai"
	@echo " - dist/linux/monolai-gui"
	@echo "=========================================="

build-windows:
	npm --prefix frontend run build
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

build-mac:
	@if [ "$$(uname -s)" = "Darwin" ]; then \
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

build: build-linux

build-all: build-linux build-windows

icons:
	./packaging/scripts/generate-icons.sh

docker-build:
	docker build -t monolai:latest -f docker/Dockerfile .

docker-up:
	docker compose up -d

docker-down:
	docker compose down

clean:
	rm -rf frontend/build frontend/.svelte-kit dist dist-windows
	cargo clean --manifest-path backend/Cargo.toml
	cargo clean --manifest-path desktop/Cargo.toml

version:
	@mkdir -p frontend/build; \
	NEW_V="$(patsubst v%,%,$(or $(VERSION_ARG),$(V)))"; \
	if [ -z "$$NEW_V" ]; then \
		NEW_V=$$(sed -n -E 's/^version = "([^"]+)"/\1/p' backend/Cargo.toml | head -n 1); \
	fi; \
	sed -i -E "0,/^version = \"[^\"]+\"/s//version = \"$$NEW_V\"/" backend/Cargo.toml; \
	sed -i -E "0,/^version = \"[^\"]+\"/s//version = \"$$NEW_V\"/" desktop/Cargo.toml; \
	sed -i -E "0,/\"version\": \"[^\"]+\"/s//\"version\": \"$$NEW_V\"/" frontend/package.json; \
	echo "export const APP_VERSION = '$$NEW_V';" > frontend/src/lib/version.ts; \
	(cd backend && cargo check --quiet); \
	(cd desktop && cargo check --quiet); \
	printf "Version synchronized to %s\n" "$$NEW_V"
