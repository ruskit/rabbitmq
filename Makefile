install-tools:
	@cargo install cargo-tarpaulin
	@cargo install cargo-audit --features=fix
	@cargo install cargo-udeps --locked
	@cargo install cargo-bloat
	@cargo install --locked cargo-pants

tests:
	cargo tarpaulin --workspace --timeout 120 --skip-clean --out Xml

audit:
	cargo audit

bloat-fn: # Get a list of the biggest functions in the release build
	cargo bloat --release -n 10

bloat-crate: #Get a list of the biggest dependencies in the release build:
	cargo bloat --release --crates