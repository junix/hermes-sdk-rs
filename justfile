arch_suffix := if arch() == "aarch64" { "arm64" } else { "x86" }
install_bin := home_directory() / "sync" / ("bin_" + arch_suffix)

build:
    cargo build --release

test:
    cargo test

install:
    @echo "hermes-sdk is a library crate, nothing to install."

demo:
    cargo run --example basic
