---
description: Compact the changelog before a release.
disable-model-invocation: true
user-invocable: true
---

Update the unreleased section of the changelog.

## Common Issues

- Entries should be headlines, not deep descriptions.
- Changes and fixes are slopped into the same group.
- Changes that aren't user-facing shouldn't be listed.

## Documenting Impact

Knowing which user-facing features were affected helps nail down which release group introduced a regression. It's worth keeping things without user-facing changes in a rewritten line enumerating the user-facing features only.

## Style

- Match the prose of previous releases.
- No em dashes.
- Strictly adhere to Keep a Changelog style.
- A release section may open with a one-line theme blurb above the first heading.
