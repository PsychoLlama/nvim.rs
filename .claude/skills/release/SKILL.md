---
description: Cut a release.
disable-model-invocation: true
user-invocable: true
---

## Preflight

- Run from a clean tree on `main`, with everything to ship already committed.
- Don't bother running tests locally. It's alpha software. Worst case, we ship a fix release.

## Tag

- Pipe the `## [Unreleased]` section into `scripts/tag-release.sh` on stdin; it becomes the tag body (the published release notes).
- The script computes `version = <UTC date>-<short sha of HEAD>`, tags HEAD, and prints the version. Read it back from there — don't recompute it.
- Do not push yet.

## Stamp the changelog

- Rename `## [Unreleased]` to `## [<version>]`, then add a fresh, empty `## [Unreleased]` above it.
- Update the link refs at the bottom: point `[Unreleased]` at `<version>...HEAD`, and add `[<version>]` comparing the previous tag to `<version>`.
- Commit as `Release <version>`. This commit lands on top of the tag — the tag names the code, this commit documents it.

## Push

- `git push origin main <version>`. This triggers the release.
- Watch the `release.yml` job. Triage if it fails.
- Use the notify skill when the release is finished.
