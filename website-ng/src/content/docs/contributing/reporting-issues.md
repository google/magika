---
title: Reporting Issues
---

This page is about opening issues. If you would rather fix something yourself, check first what pull requests we accept in [How to Contribute](/magika/contributing/how-to-contribute/#always-welcome-no-issue-needed); the [Developer Guide](/magika/contributing/developer-guide/) explains how.

A good report is short and reproducible. Pick the path that matches your case.

:::caution
Never attach files that may contain personal, confidential, or proprietary data. If a sample cannot be shared publicly, say so in the issue and email it to magika-dev@google.com instead.
:::

## Misdetections (false positives and false negatives)

Use the [misdetection template](https://github.com/google/magika/issues/new?template=misdetection.md). One issue per problem; several samples showing the same problem belong in the same issue.

Include, in this order:

1. The expected content type and the detected one.
2. The sample(s), attached (zipped) or linked.
3. The Magika version, the binding or CLI you used, and the exact command.

Misdetection reports feed directly into our evaluation and training pipeline: a sample we can reproduce is the fastest path to a fix. We take care of the fix ourselves unless we label the issue `help wanted`.

## New content types

Use the [new content type template](https://github.com/google/magika/issues/new?template=new_content_type_request.md). Tell us the format's name and extensions, what produces and consumes it, and whether a specification exists.

A few attached samples help. What helps most is pointing us to where hundreds of properly licensed samples can be obtained: public repositories, test suites, specification examples, open datasets. A request is not a commitment: supporting a content type requires samples, plus a detection rule and/or model retraining.

Once we accept a request, the content type enters the [knowledge base](https://github.com/google/magika/blob/main/assets/content_types.yaml) (label `content type to import in KB`). From that point on it is covered by the always-welcome list.

## Other bugs

Crashes, wrong API behavior, CLI or bindings problems: open a regular issue with a minimal reproducer, the Magika version, the binding, and the platform. We often fix these ourselves; the reproducer is what we need from you.

## Feature requests

Open an issue describing the problem you are trying to solve, not only the solution you have in mind. We label it `help wanted` if we agree it is worth doing and want help with it.

## Sharing samples

- Prefer links to public, properly licensed sources over attachments.
- If you attach files, zip them, keep them small, and state where they come from and under which license.
- Do not open PRs that add sample files to the repository; attach them to the issue instead.

## Security vulnerabilities

Do not open a public issue. See [Reporting Security Vulnerabilities](/magika/contributing/reporting-security-vulnerabilities/).
