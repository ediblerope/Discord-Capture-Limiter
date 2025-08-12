{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    rust-analyzer
    rustfmt
    clippy

    # Build tools (just in case some dependency needs them)
    pkg-config

    # PipeWire CLI tools (needed at runtime)
    pipewire

    # Useful development tools
    just  # task runner (optional)
    fd    # better find (optional)
  ];

  shellHook = ''
    echo "🦀 Rust development environment loaded"
    echo "🎵 PipeWire tools available: $(pw-cli --version 2>/dev/null || echo 'pw-cli not found')"
    echo ""
    echo "Try: cargo build"
    echo "Then: RUST_LOG=debug cargo run"
  '';
}
