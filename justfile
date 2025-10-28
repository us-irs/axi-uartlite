all: check build embedded clippy fmt docs coverage

clippy:
  cargo clippy -- -D warnings

fmt:
  cargo fmt --all -- --check

check:
  cargo check

embedded:
  cargo build --target thumbv7em-none-eabihf

test:
  cargo nextest r
  cargo test --doc

build:
  cargo build

docs:
  RUSTDOCFLAGS="--cfg docsrs -Z unstable-options --generate-link-to-definition" cargo +nightly doc

docs-html:
  RUSTDOCFLAGS="--cfg docsrs -Z unstable-options --generate-link-to-definition" cargo +nightly doc --open

coverage:
  cargo llvm-cov nextest

coverage-html:
  cargo llvm-cov nextest --html --open
