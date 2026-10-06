# How to Contribute

We would love to accept your patches and contributions to this project!

Check [open issues labeled as "help wanted"](https://github.com/google/magika/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22) as a starting point.

## Before you begin

### Sign our Contributor License Agreement

Contributions to this project must be accompanied by a
[Contributor License Agreement](https://cla.developers.google.com/about) (CLA).
You (or your employer) retain the copyright to your contribution; this simply
gives us permission to use and redistribute your contributions as part of the
project.

If you or your current employer have already signed the Google CLA (even if it
was for a different project), you probably don't need to do it again.

Visit <https://cla.developers.google.com/> to see your current agreements or to
sign a new one.

### Review our Community Guidelines

This project follows [Google's Open Source Community
Guidelines](https://opensource.google/conduct/).

## Contribution process

### Code Reviews

All submissions, including submissions by project members, require review. We
use [GitHub pull requests](https://docs.github.com/articles/about-pull-requests)
for this purpose.

### Adding support for new file types

Magika's file type support is model-backed. Adding a new label or improving
detection for an unsupported format usually requires more than updating metadata
files, because the model must learn to recognize representative samples of that
format.

If you want to add or improve support for a file type, please use this outline:

1. **Open or comment on an issue first.** Describe the file type, common
   extensions, MIME type if one exists, and the user impact. Include safe,
   redistributable sample files or links to public sample corpora when possible.
2. **Check whether the type is already tracked.** The content type knowledge
   base is broader than the labels supported by any single released model. A
   type may already exist in metadata but still not be supported by the current
   model.
3. **Do not rely on metadata-only changes.** Files such as content type
   metadata, model config, and model README tables describe model behavior; they
   do not by themselves teach an existing model to classify a new format.
4. **Expect model work for new labels.** New support generally needs a training
   dataset, a model update, threshold review, and evaluation against both the
   new type and nearby/confusable types.
5. **Include tests and docs with implementation changes.** When a change is
   ready, include representative test samples where licensing allows, update the
   relevant model or binding documentation, and explain any compatibility impact
   in the PR description.

For small improvements such as documentation, sample collection, or
misdetection reports, a focused issue or PR is still very helpful even if it
does not include a full model update.
