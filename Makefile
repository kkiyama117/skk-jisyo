export CARGO_NET_OFFLINE := true

.PHONY: check

check:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test
