# Contributing to Shade Stellar Contract

Thank you for your interest in contributing to this project! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/ShadeProtocol/shade-stellar-contract.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test your changes
6. Commit your changes (see [Commit Guidelines](#commit-guidelines))
7. Push to your fork: `git push origin feature/your-feature-name`
8. Open a Pull Request

## Development Setup

### Prerequisites

- Rust (stable toolchain)
- Cargo
- Soroban CLI (if needed for contract deployment)

### Building the Project

```bash
# Build all contracts
cargo build --workspace --release

# Build a specific contract
cargo build --manifest-path contracts/shade/Cargo.toml --release
```

### Running Tests

```bash
# Run all tests
cargo test --workspace --all-features

# Run tests for a specific contract
cargo test --manifest-path contracts/shade/Cargo.toml
```

### Code Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

### Linting

```bash
# Run clippy
cargo clippy --workspace --all-features -- -D warnings
```

## Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality. Install and set them up:

```bash
# Install pre-commit (requires Python)
pip install pre-commit

# Install the hooks
pre-commit install

# Run hooks manually
pre-commit run --all-files
```

The hooks will automatically:
- Check for trailing whitespace
- Ensure files end with newlines
- Validate YAML files
- Format Rust code
- Run clippy checks
- Check for merge conflicts

## Commit Guidelines

### Commit Message Format

We follow conventional commit format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Code style changes (formatting, missing semicolons, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(contract): add new token transfer function

fix(test): correct assertion in hello_world test

docs: update README with installation instructions
```

## Pull Request Process

1. **Update Documentation**: If you're adding a new feature, update the README or relevant docs
2. **Add Tests**: Ensure all new code is covered by tests
3. **Run Checks**: Make sure all tests pass and code is formatted
4. **Fill PR Template**: Complete all relevant sections in the PR template
5. **Request Review**: Request review from maintainers

### PR Checklist

Before submitting a PR, ensure:

- [ ] Code follows the project's style guidelines
- [ ] All tests pass locally
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated (if needed)
- [ ] Commit messages follow the guidelines
- [ ] PR description is clear and complete

## Code Style

- Follow Rust standard formatting (enforced by `cargo fmt`)
- Follow Rust naming conventions
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and small
- Write tests for new functionality

## Testing Guidelines

- Write unit tests for all new functions
- Test edge cases and error conditions
- Ensure test coverage doesn't decrease
- Use descriptive test names

## Reporting Issues

When reporting issues, please include:

- Description of the issue
- Steps to reproduce
- Expected behavior
- Actual behavior
- Environment details (Rust version, OS, etc.)
- Relevant logs or error messages

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Respect different viewpoints and experiences

## Questions?

If you have questions, feel free to:

- Open an issue for discussion
- Ask in pull request comments
- Contact the maintainers

Thank you for contributing! 🎉

