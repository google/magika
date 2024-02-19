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