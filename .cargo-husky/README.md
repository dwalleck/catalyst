# Cargo Husky - Automatic Git Hooks

This directory contains git hooks that are automatically installed by [cargo-husky](https://github.com/rhysd/cargo-husky) when you run `cargo build`.

## How It Works

1. **Automatic Installation**: When you run `cargo build` for the first time, cargo-husky automatically installs hooks from this directory to `.git/hooks/`
2. **Zero Configuration**: No manual setup required for new contributors
3. **Version Controlled**: Hook scripts are committed to the repo, ensuring consistency across the team

## Pre-Commit Hook

The `pre-commit` hook runs automatically before every `git commit` and performs:

- **Format Check**: `cargo fmt --all -- --check`
- **Clippy Lints**: `cargo clippy --workspace --all-features -- -D warnings`
- **Build Check**: `cargo build --workspace --all-features`

If any check fails, the commit is blocked and you'll see an error message explaining what went wrong.

## Bypassing Hooks (Emergency Only)

If you absolutely need to bypass the pre-commit hook:

```bash
git commit --no-verify
```

**⚠️ Warning:** Only use this in emergencies. The hooks ensure code quality and prevent broken code from being committed.

## Manual Installation (Existing Repos)

If you cloned the repo before hooks were added, run:

```bash
cargo build
```

This will trigger cargo-husky to install the hooks.

## Modifying Hooks

To modify the pre-commit checks:

1. Edit `.cargo-husky/hooks/pre-commit`
2. Commit the changes
3. Other developers will get the updated hooks on their next `cargo build`

## Troubleshooting

**Hooks not running?**
- Verify hooks were installed: `ls -la .git/hooks/pre-commit`
- Reinstall with: `cargo clean && cargo build`

**Hook failing incorrectly?**
- Run checks manually: `./.cargo-husky/hooks/pre-commit`
- Check the error output for specific failures

**Need to update hooks after pulling?**
- Run `cargo build` to reinstall updated hooks

## Why cargo-husky?

- ✅ **Rust-native**: No Python, Go, or other runtime dependencies
- ✅ **Automatic**: Hooks install on first build
- ✅ **Simple**: Just add to dev-dependencies
- ✅ **Fast**: Minimal overhead, pure Rust implementation
- ✅ **Versioned**: Hook scripts committed to repo

## References

- [cargo-husky GitHub](https://github.com/rhysd/cargo-husky)
- [Git Hooks Documentation](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)
