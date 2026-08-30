# Homebrew tap

The project tap publishes a source-building formula before `code-a-cv` is eligible for Homebrew Core. The GitHub repository is `voxvanhieu/homebrew-tap`, and Homebrew addresses it as `voxvanhieu/tap`.

## Create the tap repository

This is a one-time setup. Create an independent repository, not a fork of Homebrew Core:

```console
$ brew tap-new voxvanhieu/tap
$ tap_dir="$(brew --repository voxvanhieu/tap)"
$ gh repo create voxvanhieu/homebrew-tap --public --source "${tap_dir}" --remote origin --push
```

Keep the workflows created by `brew tap-new`. They test formula pull requests and can publish bottles with `brew pr-pull`.

## Configure repository access

Create a fine-grained GitHub personal access token that can access only `voxvanhieu/homebrew-tap`. Grant it read and write access to repository contents and pull requests.

Add the token to `voxvanhieu/code-a-cv` as the Actions secret `HOMEBREW_TAP_TOKEN`. The `Publish Homebrew tap formula` workflow uses it to push a version branch and open a pull request. It does not merge the pull request.

## Publish a release

Publish the stable GitHub release first. The release tag must match the `cac` package version and the release must contain `source.tar.gz`.

Open the Actions page for `voxvanhieu/code-a-cv`, run `Publish Homebrew tap formula`, and enter the stable `vX.Y.Z` tag. The workflow:

1. Rejects drafts, prereleases, malformed tags, and version mismatches
2. Generates a formula from the release source archive and its SHA-256 checksum
3. Builds, tests, audits, and styles the formula on Linux and macOS
4. Pushes `code-a-cv-X.Y.Z` to `voxvanhieu/homebrew-tap`
5. Opens a pull request without merging it

Review the formula and the workflow results. Use the tap repository's generated `brew pr-pull` workflow with the pull request number and reviewed head SHA to publish bottles. If bottles are not published, users build the formula from source.

Install the published formula with:

```console
$ brew install voxvanhieu/tap/code-a-cv
```

Using the fully qualified name trusts only this formula instead of the entire third-party tap.

## Update a release

Run `Publish Homebrew tap formula` once for each stable release. Do not reuse a version tag or replace a published source archive because the formula pins its SHA-256 checksum.

The workflow stops if `tap_migrations.json` says that `code-a-cv` moved to `homebrew/core`. This prevents a later release from recreating the tap formula after migration.

## Migrate to Homebrew Core

A tap formula and a Core formula with the same name cannot be installed side by side. When the initial Homebrew Core pull request is ready, create a coordinated tap pull request that deletes `Formula/code-a-cv.rb` and adds:

```json
{
  "code-a-cv": "homebrew/core"
}
```

Use the commit message `code-a-cv: migrate to homebrew/core`. Link the tap and Core pull requests and merge the migration only when the Core formula is available.

Keep `voxvanhieu/homebrew-tap` after migration so `brew update` can move existing installations to Core. Update the installation command in `README.md` to:

```console
$ brew install code-a-cv
```

Future releases then use `brew bump-formula-pr` for the official Core formula. Do not run `Publish Homebrew tap formula` again.
