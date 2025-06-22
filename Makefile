build:
	cargo build --release
run:
	cargo run
fmt:
	cargo fmt
test:
	cargo test
debug:
	RUST_BACKTRACE= cargo run