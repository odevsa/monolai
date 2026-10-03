ifeq (version,$(firstword $(MAKECMDGOALS)))
  VERSION_ARG := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(eval $(VERSION_ARG):;@:)
endif

.PHONY: help all dev dev-host dev-frontend dev-backend build build-server build-desktop build-all install clean version docker-build docker-up docker-down

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
	@printf "%-18s %s\n" "build" "Build both server and desktop release binaries"
	@printf "%-18s %s\n" "build-server" "Build frontend and release server binary (monolai)"
	@printf "%-18s %s\n" "build-desktop" "Build native desktop GUI binary (monolai-gui)"
	@printf "%-18s %s\n" "docker-build" "Build Docker image locally"
	@printf "%-18s %s\n" "docker-up" "Start services with docker compose"
	@printf "%-18s %s\n" "docker-down" "Stop services with docker compose"
	@printf "%-18s %s\n" "clean" "Remove build artifacts"
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

build-desktop:
	cargo build --manifest-path desktop/Cargo.toml --release

build: build-server build-desktop

build-all: build

docker-build:
	docker build -t monolai:latest -f docker/Dockerfile .

docker-up:
	docker compose up -d

docker-down:
	docker compose down

clean:
	rm -rf frontend/build frontend/.svelte-kit
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
