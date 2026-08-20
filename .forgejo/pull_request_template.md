<!-- Please put the pull request description above. -->
<!-- dprint-ignore-start -->

## Checklist

The [CONTRIBUTING.md](https://codeberg.org/GramEditor/gram/src/branch/main/CONTRIBUTING.md) contains information that will be helpful to first time contributors.

### Compliance

- [ ] I have read the [Gram Code of Conduct](https://codeberg.org/GramEditor/gram/src/branch/main/CODE_OF_CONDUCT.md) and agree with them.

### Tests for Rust changes

<!-- Can be removed for non-Rust changes. -->

- I ran...
  - [ ] `cargo run --profile dev` to check for issues with app building.
  - [ ] `cargo fmt --all` to check formatting.
  - [ ] `./script/clippy` to check linting.

### Documentation

- [ ] I created a commit [to the documentation](https://codeberg.org/GramEditor/gram/src/branch/main/docs) to explain to Gram users how to use this change.
- [ ] I did not document these changes and I do not expect someone else to do it.

### Release notes

- [ ] This change will be noticed by a Gram user (feature, bug fix, performance, etc.). I suggest to include a release note for this change.
- [ ] This change is not visible to a Gram user (refactor, dependency upgrade, etc.). I think there is no need to add a release note for this change.

<!-- dprint-ignore-end -->
