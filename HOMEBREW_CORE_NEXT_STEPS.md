# Homebrew Core next steps

Code a CV is not yet eligible for an owner-submitted Homebrew Core formula. Homebrew requires the upstream repository to have at least one of:

* 225 legitimate GitHub stars
* 90 legitimate GitHub forks
* 90 legitimate GitHub watchers

The `Prepare initial Homebrew Core formula` workflow checks these thresholds before it starts the Homebrew builds. Do not bypass this check or manufacture repository activity.

## Before the project is eligible

Publish each stable release to `voxvanhieu/homebrew-tap` with the `Publish Homebrew tap formula` workflow. Users install the source-building formula with:

```console
$ brew install voxvanhieu/tap/code-a-cv
```

See `HOMEBREW_TAP.md` for the one-time repository setup, release procedure, and migration plan.

Do not create or submit a Homebrew Core formula branch yet. Homebrew will reject the pull request while the project does not meet its acceptance policy.

## After the project is eligible

1. Publish a stable `vX.Y.Z` GitHub release from the latest validated `main` commit
2. Open the GitHub Actions page for this repository
3. Run `Prepare initial Homebrew Core formula` with the stable release tag
4. Wait for formula generation, Linux validation, macOS validation, and strict audit to pass
5. Confirm that the workflow pushed `code-a-cv-X.Y.Z` to `voxvanhieu/homebrew-core`
6. Review the branch diff before opening a pull request

The workflow does not open the upstream pull request automatically.

## Validate the Homebrew Core branch

Use the Homebrew Core checkout at `~/Workspaces/homebrew-core`:

```console
$ cd ~/Workspaces/homebrew-core
$ git fetch voxvanhieu
$ git switch --track voxvanhieu/code-a-cv-X.Y.Z
$ git pull --ff-only voxvanhieu code-a-cv-X.Y.Z
$ HOMEBREW_NO_INSTALL_FROM_API=1 brew install --build-from-source code-a-cv
$ brew test code-a-cv
$ brew audit --new --strict code-a-cv
$ brew style Formula/c/code-a-cv.rb
```

Confirm that no existing open or closed Homebrew Core pull request already covers `code-a-cv` before submitting.

## Open the Homebrew Core pull request

Open a pull request from `voxvanhieu:code-a-cv-X.Y.Z` to `Homebrew/homebrew-core:main`. Keep it limited to the new formula and retain the generated commit message:

```text
code-a-cv X.Y.Z (new formula)
```

Complete the Homebrew pull request template truthfully. Disclose that AI assisted with the formula and workflow, and state which validation was performed by AI and which validation you performed personally. Answer all maintainer questions and review comments yourself without AI assistance.

## Migrate the tap formula

Coordinate the Core pull request with a pull request in `voxvanhieu/homebrew-tap`. The tap pull request must delete `Formula/code-a-cv.rb` and add this entry to `tap_migrations.json`:

```json
{
  "code-a-cv": "homebrew/core"
}
```

Link the two pull requests and merge the migration only when the Core formula is available. Do not delete the tap repository because existing installations need its migration metadata during `brew update`.

After the formula is accepted, use `brew bump-formula-pr` for later version updates instead of running either initial-formula workflow.

See the [Homebrew Package Acceptance Policy](https://docs.brew.sh/Package-Acceptance-Policy) and [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook) for the current requirements.
