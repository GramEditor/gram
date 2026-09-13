<!-- Please put the pull request description above. -->
<!-- dprint-ignore-start -->

## Checklist

The [CONTRIBUTING.md](https://codeberg.org/GramEditor/gram/src/branch/main/CONTRIBUTING.md) contains helpful information for first-time contributors.

<!--

NOTE: We require commits to be signed. See: <https://git-scm.com/book/en/v2/Git-Tools-Signing-Your-Work>.
You can use either a GPG key or a SSH key to sign your commits. Ideally you should add your
key to your Codeberg account so that it can verify the commits.

- SSH: <https://docs.codeberg.org/security/ssh-key>.
- GPG: <https://docs.codeberg.org/security/gpg-key>

-->

### Compliance

- [ ] I have read and agree to the [Gram Code of Conduct](https://codeberg.org/GramEditor/gram/src/branch/main/CODE_OF_CONDUCT.md).

### Rust changes

<!-- You can skip and remove this section if your changes do not involve Rust code. -->

- [ ] `cargo check --workspace --all-targets --locked` passes.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `./script/clippy` passes.
- [ ] `cargo nextest run --workspace --locked` passes (optional).

### Documentation

- [ ] I have updated the [documentation](https://codeberg.org/GramEditor/gram/src/branch/main/docs) to reflect this change.
- [ ] This change does not require documentation updates.

### Release notes

- [ ] This change is user-facing (feature, bug fix, performance, etc.). A release note should be added.
- [ ] This change is internal (refactor, dependency upgrade, CI, etc.). No release note is needed.

<!-- dprint-ignore-end -->
