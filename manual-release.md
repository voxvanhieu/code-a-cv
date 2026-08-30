# Manual release

This guide creates a release from the latest commit on `main` without AI assistance. Replace `0.1.1` with the version being released.

## 1. Update local `main`

```console
$ git switch main
$ git pull --ff-only origin main
$ git status
```

Continue only when `git status` reports a clean working tree.

## 2. Prepare the version and changelog

Update `workspace.package.version` in `Cargo.toml` and add the release date and changes to `CHANGELOG.md`. Update `Cargo.lock` after changing the workspace version:

```console
$ cargo check --workspace --locked
```

If this command reports that the lock file needs to be updated, run `cargo check --workspace` once and then continue with the checks below.

## 3. Validate and commit the release

```console
$ cargo fmt --all --check
$ cargo test --workspace --all-features --locked
$ cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
$ dist plan
$ git add Cargo.toml Cargo.lock CHANGELOG.md
$ git commit -m "0.1.1"
$ git push origin main
```

Wait for the GitHub Actions checks on `main` to pass before creating the tag.

## 4. Create and push the release tag

Confirm that `HEAD` is the commit that passed CI, then create a new tag:

```console
$ git status
$ git log -1 --oneline
$ git tag v0.1.1
$ git push origin v0.1.1
```

Never reuse or move an existing version tag. The tag version must match `workspace.package.version` in `Cargo.toml`.

## 5. Watch and verify the release

Find the run started by the tag, copy its database ID, and watch it:

```console
$ gh run list --workflow release.yml --limit 1
$ gh run watch RUN_ID
$ gh release view v0.1.1
```

The release workflow creates the GitHub release and uploads its source archives, binary archives, checksums, and installers.

## 6. Update Homebrew

Before the formula is accepted into Homebrew Core, open the GitHub Actions page and run `Publish Homebrew tap formula` with the new stable tag. Review the pull request opened in `voxvanhieu/homebrew-tap`, wait for its checks, and use the tap's `brew pr-pull` workflow to publish bottles.

Run the following command only after the formula has been accepted into Homebrew Core:

```console
$ brew bump-formula-pr \
    --version=0.1.1 \
    --url="https://github.com/voxvanhieu/code-a-cv/releases/download/v0.1.1/source.tar.gz" \
    code-a-cv
```

For the first Homebrew Core submission, run the `Prepare initial Homebrew Core formula` workflow from the GitHub Actions page after the GitHub release succeeds. Review the branch pushed to `voxvanhieu/homebrew-core`, then open the pull request to `Homebrew/homebrew-core` manually.
