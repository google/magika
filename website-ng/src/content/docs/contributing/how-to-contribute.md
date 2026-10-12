---
title: How to Contribute
---

Thanks for your interest in Magika! This page explains what we accept and why. It is short on purpose: please read it before opening an issue or a pull request.

LLMs and coding agents have changed the balance between writing and reading: producing a patch or a long issue now takes minutes, while reviewing it still takes a human's time. These guidelines are how we adapt to that. They are not set in stone; we will adjust them as we see how things go.

## TL;DR

- We review pull requests only for issues labeled [`help wanted`](https://github.com/google/magika/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22) and for the short [always-welcome list](#always-welcome-no-issue-needed) below. For anything else, open a concise issue first.
- Use whatever tools you like, AI included. You own every line you submit, you can explain it, and you are the one who talks to us.
- Keep it short. One PR, one problem, a few lines of description. We read everything by hand.
- Low-effort submissions are closed without discussion. Repeat offenders are blocked.
- No claiming: the first good PR for an issue wins.

## How we work

Magika is maintained by a small team with a large surface: models, detection rules, a Rust core, bindings for several languages, and this website. Review time is our scarcest resource, so we decide what we want help with and say so explicitly with the `help wanted` label.

The most valuable contribution is a good issue. If we agree that a PR would help, we add the label and PRs are welcome. Otherwise we fix it ourselves.

## What we accept

1. PRs for open issues labeled `help wanted`. Reference the issue in the description (`Fixes #N`).
2. PRs for the always-welcome list below.
3. Anything else is closed with a pointer to this page, usually without further comment. This is not a judgment on the work; it is how we keep review time for the things we asked for.

### Always welcome, no issue needed

The common thread: small, self-contained, and easy to verify.

- **Documentation fixes**: typos, grammar, dead links, outdated snippets. One PR per logical fix, not one per typo.
- **Metadata fixes for content types already in the [knowledge base](https://github.com/google/magika/blob/main/assets/content_types.yaml)**: description, extensions, MIME type. Cite a source, run `just sync-kb`, and commit the generated files. See the [developer guide](/magika/contributing/developer-guide/#updating-the-content-types-knowledge-base).
- **Detection rules for content types already in the knowledge base**: new rules or fixes to existing ones, with the evidence described in the [developer guide](/magika/contributing/developer-guide/#improving-detection-with-rules). If the content type is not in the knowledge base yet, [request it](/magika/contributing/reporting-issues/#new-content-types) first.
- **Fixes that turn a red CI check on `main` green again**. Link the failing run.

Not on the list, so open an issue first: new features, new bindings, refactors, sample files, dependency or version bumps, and bug fixes in the CLI or bindings (we need the reproducer anyway).

## What a good issue looks like

What happened, what you expected, and how to reproduce it: Magika version, command, and the file (attached or linked), in that order. No PII, no walls of text, no pasted agent transcripts. See [Reporting Issues](/magika/contributing/reporting-issues/) for misdetections, new content types, and other bugs.

## What a good PR looks like

Write the PR you would want to review:

- One PR, one problem. Linked to a `help wanted` issue, or naming the always-welcome item it falls under.
- A description of a few lines: what, why, how you tested it. If the description is longer than the diff, something is off.
- CI green, generated files regenerated (`just sync-kb`, `./sync.sh`, ...).
- No drive-by refactors, renames, formatting churn, or version bumps.

## AI, agents, and accountability

We do not ask whether or how you used AI, and it does not change how we review. What we do ask:

- You have read and understood every line you submit, and you can discuss it.
- When we reply, we expect a human to read and answer. If we wanted an agent's answer, we would ask our own; there is no need to relay one through GitHub.
- By opening an issue or a PR, you take responsibility for it.

## What gets closed, and when we block

- Low-effort submissions: PRs not linked to a `help wanted` issue or to the always-welcome list, generic "improvements", descriptions that do not match the diff, agent output pasted as-is. Effort is not length.
- Anything that looks like contribution farming goes against the spirit of this project and is closed on sight.
- Repeat offenders are blocked from the repository.

## Before your first PR

- Sign the [Contributor License Agreement](https://cla.developers.google.com/about) (CLA). You only need to do it once; check your status at [cla.developers.google.com](https://cla.developers.google.com/).
- This project follows [Google's Open Source Community Guidelines](https://opensource.google/conduct/).
- All submissions, including those by project members, go through code review on GitHub.

The repository's [`CONTRIBUTING.md`](https://github.com/google/magika/blob/main/CONTRIBUTING.md) carries the same summary and the legal notes.
