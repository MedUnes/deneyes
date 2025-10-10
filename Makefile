.PHONY: build run fmt lint test release api deb dashboard-install dashboard-build clean

build:
	cargo build

release:
	cargo build --release

run:
	cargo run -- dns

api:
	cargo run -- api

fmt:
	cargo fmt

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test

clean:
	cargo clean

# Build the Debian package into target/debian
# Requires dpkg-deb to be available on the system.
deb:
	scripts/build-deb.sh

# Frontend helpers
# Install dashboard dependencies (requires npm)
dashboard-install:
	cd web/dashboard && npm install

# Build the dashboard production bundle
# Requires dependencies to be installed first.
dashboard-build:
	cd web/dashboard && npm run build
