# Contributing to Solana Stablecoin Standard

Thank you for your interest in contributing to the Solana Stablecoin Standard (SSS). This framework aims to provide robust, production-ready stablecoin infrastructure for the Solana ecosystem.

## Code of Conduct

This project and everyone participating in it is governed by the [Solana Stablecoin Standard Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to security@superteambrazil.com.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues as it might be already reported. When you are creating a bug report, please include as many details as possible:

* **Use a clear and descriptive title** for the issue to identify the problem.
* **Describe the exact steps which reproduce the problem** in as many details as possible.
* **Explain which behavior you expected to see and why.**
* **Include screenshots** if applicable.

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

* **Use a clear and descriptive title.**
* **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
* **Explain why this enhancement would be useful** to most users.

## Development Workflow

1. **Fork and Clone**: Fork the repository and clone it locally.
2. **Install Dependencies**: Run `yarn install` to install all necessary dependencies.
3. **Branch Naming**: Create a branch using the format `feature/description` or `fix/issue-description`.
4. **Commit Messages**: Use conventional commits (e.g., `feat: add oracle module`, `fix: correct pda derivation`).
5. **Testing**: All changes must be covered by tests. Run the test suite before submitting.
   - Program tests: `cargo test -p sss-token`
   - Integration tests: `anchor test`
6. **Code Style**: 
   - Rust: Run `cargo fmt` and ensure `cargo clippy -- -W clippy::all` passes with no warnings.
   - TypeScript: Follow the strict compiler options defined in `tsconfig.json`.

## Security Guidelines

Security is the highest priority for this repository. If you are proposing a code change:
- **No `unwrap()` or `expect()`**: Always use proper error handling (`Result`) in program code.
- **Arithmetic**: Always use checked math operations (`checked_add`, `checked_sub`).
- **Account Validation**: Ensure strict validation on all accounts, especially signers and PDAs.
- **Access Control**: Changes to role management or authorities must undergo strict scrutiny.

If you discover a security vulnerability, please do not open a public issue. Email security@superteambrazil.com directly.

## Pull Request Process

1. Ensure your PR is focused on a single issue or feature.
2. Fill out the Pull Request template entirely.
3. Keep PRs as small as possible to facilitate review.
4. CI checks must pass before a PR can be merged.
5. A core maintainer will review your code. Address any feedback promptly.
6. Once approved, your PR will be merged by a maintainer.

## Architecture

Before contributing, please read `docs/ARCHITECTURE.md` to understand the three-layer model (Base SDK, Modules, Standard Presets). Ensure your feature aligns with this modular approach.
