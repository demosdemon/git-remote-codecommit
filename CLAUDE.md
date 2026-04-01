# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A git remote helper for AWS CodeCommit written in Rust. It allows using `codecommit://` URLs with git by generating AWS SigV4-signed HTTPS URLs and delegating to `git remote-https`. This is a statically-linked Rust alternative to the original Python `git-remote-codecommit`.

## Build Commands

```bash
cargo build                       # Debug build
cargo build --profile=release-lto # Release build with LTO
cargo test                        # Run all tests
cargo clippy                      # Lint (pedantic warnings enabled)
cargo +nightly fmt                # Format code
cargo +nightly fmt -- --check     # Check formatting without modifying
```

Single test: `cargo test <test_name>` or `cargo test -p git-remote-codecommit <test_name>`

## Architecture

The binary is invoked by git as a remote helper for `codecommit://[profile@]repo[::region]` URLs.

**Core flow** (`crates/git-remote-codecommit/src/main.rs`):
1. Parse CLI args (remote name + URI) via clap
2. Parse the `codecommit://` URI (`uri/`)
3. Load AWS credentials and region via `sdk_context.rs`
4. Build an AWS SigV4-signed HTTPS URL:
   - `hostname.rs` → infer CodeCommit endpoint
   - `credential_scope.rs` → build credential scope
   - `canonical_request.rs` → construct canonical request
   - `string_to_sign.rs` → create string to sign
   - `username.rs` → generate HTTP Basic auth username from signature
5. Exec `git remote-https` with the signed URL (Unix: `execvp`, Windows: subprocess with Ctrl-C handler)

**Compiler feature probing** (`build.rs`, `src/nightly/`): The build script probes for unstable Rust features (`bool_to_result`, `windows_process_exit_code_from`) and sets cfg flags so the code can use them when available.

## Key Constraints

- **MSRV**: 1.91.1 (enforced in `clippy.toml` and CI)
- **Edition**: Rust 2024
- **Clippy**: Pedantic warnings enabled at workspace level
- **Rustfmt**: Nightly features used (format_macro_matchers, group_imports, imports_granularity)
- **Dual license**: MIT OR Apache-2.0
