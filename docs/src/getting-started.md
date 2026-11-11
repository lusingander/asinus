# Getting Started

Use a True Color terminal with at least 60 columns and 22 rows. See [Compatibility](reference/compatibility.md) for platform and clipboard requirements.

## Installation

### Cargo

Install from [crates.io](https://crates.io/crates/asinus) with Rust 1.89 or later:

```sh
cargo install --locked asinus
```

### Homebrew (macOS)

```sh
brew install lusingander/tap/asinus
```

The formula is maintained in [lusingander/homebrew-tap](https://github.com/lusingander/homebrew-tap).

### Release Binaries

Download an archive from [GitHub Releases](https://github.com/lusingander/asinus/releases), extract it, and place the `asinus` executable on your `PATH`.

| Operating system | Architecture | Target |
| --- | --- | --- |
| macOS | Apple Silicon | `aarch64-apple-darwin` |
| macOS | Intel | `x86_64-apple-darwin` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` or `aarch64-unknown-linux-musl` |
| Linux | x86-64 | `x86_64-unknown-linux-gnu` or `x86_64-unknown-linux-musl` |

## First Steps

Start Asinus:

```sh
asinus
```

1. Use `↑` / `↓` to select a theme.
2. Press `p` to pin it, then select another theme and press `c` to compare them.
3. Use `[` / `]` to switch preview samples, or `/` to search.
4. Press `F1` for help and `Ctrl-C` to quit.

Continue with [Explore & Compare](features/explore-compare.md) or try [your own preview sample](features/explore-compare.md#custom-samples). See [Keybindings](reference/keybindings.md) for all controls.

Run `asinus --help` for command line help and `asinus --version` to check the installed version.
