# Contributing to graphfinder

Thanks for your interest in contributing! This project is a graph traversal &
pathfinding library (informed and uninformed search) with a Rust core and a
Python API, built for teaching. Contributions of all kinds are welcome: bug
reports, documentation, new algorithms, new domains/heuristics, visualization
and performance work.

## Reporting bugs and requesting features

Please open an issue at <https://github.com/graphfinder/graphfinder.github.io/issues> and
include, where relevant:

- what you expected to happen and what happened instead;
- a minimal reproducible example (Python or Rust);
- your OS, Rust version (`rustc --version`) and Python version;
- the `graphfinder` / `graphfinder-core` version.

## Development setup

You need [Rust](https://rustup.rs) and Python ≥ 3.9.

```bash
# Rust core
cargo test -p graphfinder-core      # tests + doctests
cargo clippy -p graphfinder-core --all-targets -- -D warnings
cargo fmt

# Python (Rust core via maturin)
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest matplotlib
maturin develop --release
pytest -q tests/
```

After ANY change in `crates/graphfinder-core` or `crates/graph-py`, re-run
`maturin develop` so Python sees it.

## Conventions

- **Language:** all code, comments and docs are in English.
- **Teaching first:** every new algorithm/domain ships a runnable example and a
  test that asserts its defining property (see `CLAUDE.md`).
- **Reproducibility:** randomized instances take a `seed`; keep tie-breaking
  deterministic.
- **CI must pass:** `cargo fmt --check`, `clippy -D warnings`, and the test
  suites (Rust + Python).

See `CLAUDE.md` for the architecture and `ROADMAP.md` for planned work.

## Releasing a new version

Releases are tag-driven. Pushing a `v*` tag triggers three workflows:
`release-pypi` (PyPI), `release-crates` (crates.io) and `release-github`
(GitHub Release). Both package publishes use **Trusted Publishing (OIDC)** —
there are **no API tokens** stored in the repo.

Steps:

1. Bump the version in **three** files (keep them in sync):
   - `Cargo.toml` → `[workspace.package] version`
   - `pyproject.toml` → `[project] version`
   - `CITATION.cff` → `version` and `date-released`
2. Make sure CI is green and the crate packages cleanly:
   ```bash
   cargo publish --dry-run -p graphfinder-core
   ```
3. Commit the bump, then tag and push:
   ```bash
   git commit -am "Release vX.Y.Z"
   git tag vX.Y.Z
   git push origin main vX.Y.Z
   ```
4. Watch the runs: `gh run list`. The PyPI job builds wheels (Linux/macOS/
   Windows) + sdist and publishes; the crates job mints a short-lived token via
   OIDC and runs `cargo publish`.

Versions are immutable on both registries — a number can never be reused, so
double-check before tagging.

### Trusted Publishing setup (one-time, already configured for 0.1.0)

- **PyPI:** a publisher at <https://pypi.org/manage/account/publishing/> →
  project `graphfinder`, owner `graphfinder`, repo `graphfinder.github.io`,
  workflow `release-pypi.yml`.
- **crates.io:** on
  <https://crates.io/crates/graphfinder-core/settings> → Trusted Publishing →
  owner `graphfinder`, repo `graphfinder.github.io`, workflow
  `release-crates.yml`.

## License

By contributing, you agree that your contributions will be licensed under the
MIT License.
