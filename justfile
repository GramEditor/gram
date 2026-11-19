debug:
  GRAM_STATELESS=1 cargo run

fmt:
  cargo fmt --all

check:
  cargo fmt --all -- --check
  cargo check --workspace --all-targets --locked
  ./script/clippy

lint:
  ./script/clippy

test:
  cargo nextest run --workspace --no-fail-fast --locked

clean:
  cargo clean
