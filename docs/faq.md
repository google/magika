# Frequently Asked Questions

## Q: Why does Magika support "only" ~100 content types and not many more?

Because we needed to start from somewhere. Magika is based on a new approach, and at first we did not know whether it would work or not. It was prohibitively complex to aim to support all content types from the very beginning, and we aimed at selecting at least 100 content types (we settled with 110+). Which ones? The ones that seemed most relevant for most use cases (but, still, we miss many more!). Now that we know this approach works, we will be looking at improving content types coverage for the next iterations.


## Q: Why does not Magika support content type X or Y?

See previous question.

But please open GitHub issues on what you want! Getting this sort of feedback was one main reason to open source an early version.


## Q: What is the use case for the javascript package?

The main client we expect people to use for this release is the Python client and Python API. The javascript package, based on a TFJS version of the same model, was developed for our [web demo](https://google.github.io/magika/), which allows users to test Magika and report feedback without installing anything. The demo also showcases on-device capabilities. The javascript package could also be useful for integrations that require javascript bindings. For now it is not envisioned to be used as a standalone command line (the model loading phase is quite slow), but it could be useful for those deployments where you can load the model once, and keep using it for many inferences.


## Q: Where can I find more details about all this?

We are releasing a paper later this year detailing how the Magika model was trained and the specifics about the model itself. We will also open source other components of this project (e.g., the keras model Python code). Stay tuned!


## Q: The inference time is ~5ms but the Python CLI takes a few hundred ms to bootstrap?

Yes, but this is because the Python CLI needs to load the Python interpreter and various libraries, plus the model. For the future, we are considering other options (e.g., a Rust client).

In the meantime, we believe the current release is already good enough for many use cases, including scanning thousands of files: you can pass them all as arguments in one single invocation, and the Python client (and API) will internally load the model only once and use batching to achieve fast inference speeds.


## Q: Magika returns the content type in a number of formats, which one should I use?

If Magika's output is for human consumption, then the default verbose textual description should be fine (but, if you want, you can change the output format with CLI options, e.g., `--label`, `--jsonl`, ...).

However, if you are using Magika for automated pipelines, we strongly suggest to use the simple textual label,`ct_label` (either by using `--label`, or by using `--jsonl` and parse out the `ct_label` field). Other verbose output (e.g., `file`'s magic) or MIME types have proved to be very painful to work with. See the following Q/A for more context.


## Q: What is the problem with relying on type detectors' verbose textual description for automated workflows?

Content type detectors' textual descriptions often change without considerations for backward compatibility issues.

See for example [this commit](https://github.com/file/file/commit/a2756aa50fdf7d87ebb14002ffd7609373ea6839) from `file`, in which the textual output for `javascript` was slightly changed (from `Node.js script text executable` to `Node.js script executable`).

These textual representations could also slightly differ even for the same content type, thus making the normalization process rather tedious and error-prone. For example, these are different variations on how an XML file can be recognized by `file`: `XML document`, `XML 1.0 document`, or `XML 1.0 document text`.

Thus, this type of output is useful for human consumption, but it does not appear suitable for automated workflows.


## Q: What is the problem with relying on MIME types for automated workflows?

Despite the popularity of using MIME types to label content types, we have run into problems when performing our evaluation of existing tools.

We found that the same given content type can be associated to multiple MIME types (e.g., `application/xml`, `text/xml`). The full mapping is often not available, and can change with time. This makes the mapping task tedious and error-prone.

Another problem relates to content types that were initially not officially registered and that they were registered only later on. For example, Markdown was initially associated to `text/x-markdown` (the `x-` prefix is used for unofficial MIME types), but, later on, Markdown was registered and associated to `text/markdown`. The problem is that, when (and if) existing tools update their MIME types database, existing automated workflows would break.

One last example that may unexpectedly break backward compatibility is that tools such as `file` may change the outputted MIME type even for very popular content types. For example, recent versions of `file` started to output `application/vnd.microsoft.portable-executable` as MIME type for Windows PE executable files; previous versions were outputting `application/x-dosexec` or similar variations.

The changing nature of MIME types and the relatively slow process of registration of new ones make MIME types challenging to be used for automated workflows.

We believe simple textual labels are better for automated workflows.

[Do you disagree? Please open a GitHub issue and let's discuss! We are happy to hear feedback.]


## Q: Does Magika return additional metadata other than a content type label?

Not at the moment.

For example, Magika classifies an ELF binary as `elf` and a Windows PE file as `pebin`, but it does not go into details about what specific kind of ELF it is dealing with (e.g., statically vs. dynamically linked).

This version of Magika was developed as a filter for automated pipelines, and the high-level content type label is usually what is needed.

We have plans to explore whether our approach can be extended to return finer-grained information, but this is currently not supported.
