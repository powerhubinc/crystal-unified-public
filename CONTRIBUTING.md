# Contributing to Crystal Unified

Thank you for your interest in contributing to Crystal Unified!

## Before You Contribute

### Contributor License Agreement (CLA)

**All contributors must agree to our [Contributor License Agreement (CLA)](LICENSES/CLA.md)
before any contribution can be accepted.**

By submitting a pull request, you agree that:
- You have read and agree to the CLA
- You have the right to submit the contribution
- Your contribution becomes the property of Powerhub Inc.

### Sign Your Commits

All commits must include a sign-off line:

```
Signed-off-by: Your Name <your.email@example.com>

I have read and agree to the Crystal Unified Contributor License Agreement.
```

Use `git commit -s` to automatically add the sign-off.

---

## How to Contribute

### Reporting Issues

1. Search existing issues to avoid duplicates
2. Use a clear, descriptive title
3. Provide detailed steps to reproduce
4. Include system information (OS, Rust version, etc.)

### Submitting Changes

1. **Fork** the repository
2. **Create a branch** for your changes
3. **Write tests** for new functionality
4. **Ensure all tests pass**: `cargo test`
5. **Check formatting**: `cargo fmt`
6. **Check lints**: `cargo clippy`
7. **Sign your commits** (see above)
8. **Submit a pull request**

### Code Style

- Follow Rust standard conventions
- Use `cargo fmt` before committing
- All public APIs must be documented
- Keep functions focused and small
- Write meaningful commit messages

### Pull Request Guidelines

- One feature/fix per PR
- Update documentation if needed
- Add tests for new functionality
- Ensure CI passes
- Respond to review feedback promptly

---

## What We Accept

We welcome:
- Bug fixes
- Performance improvements
- Documentation improvements
- Test coverage improvements

We may not accept:
- Breaking API changes without discussion
- Features that don't align with project goals
- Changes without tests

---

## Code of Conduct

All contributors must follow our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## Questions?

- **General questions:** Open an issue
- **Security issues:** See [SECURITY.md](SECURITY.md)
- **Licensing questions:** licensing@powerhub.inc

---

Copyright (c) 2026 Powerhub Inc. All rights reserved.
