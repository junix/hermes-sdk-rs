os_suffix := if os() == "macos" { "macos" } else { "linux" }
arch_suffix := if arch() == "aarch64" { "arm64" } else { "x86" }
install_bin := env("SYNC_BIN_DIR", home_directory() / "sync" / (os_suffix + "-" + arch_suffix + "-bin"))

build:
    cargo build --release

test:
    cargo test

install:
    @echo "hermes-sdk is a library crate, nothing to install."

demo:
    cargo run --example basic
