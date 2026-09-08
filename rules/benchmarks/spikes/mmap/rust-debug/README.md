# Rust debugger startup findings

`rust-lldb` stopped the optimized binary at `_dyld_start`, Objective-C initialization,
and the first Rust main statement (`rust/cli/src/main.rs:351`). The debug build uses
release optimization/LTO and adds `-C debuginfo=2` to the CLI compilation. No timings
are inferred from breakpoint pauses. The earlier Hyperfine measurements remain the
performance evidence.

The actual pre-main initializer stack includes:

```text
dyld::start
  dyld4::prepare
    dyld4::APIs::runAllInitializersForMain
      dyld4::PrebuiltLoader::runInitializers
        dyld4::Loader::findAndRunAllInitializers
          libSystem_initializer
            libdispatch_init
              _os_object_init
                _objc_init
```

At the first Rust main statement, LLDB reports **347 images**, including Metal,
Foundation, CoreServices, IOAccelerator and GPU compiler utilities. This total also
includes ordinary OS libraries; it is not a count of exclusively ML dependencies.
Vectorscan has not yet been loaded at that stop. The process subsequently returns
the same dbase classification as the uninstrumented binary and exits successfully.

The code dependency is explicit:

```text
magika-cli -> magika -> magika-tract-runtime -> tract-metal
```

`rust/lib/Cargo.toml:35` makes the inference runtime unconditional.
`rust/tract-runtime/Cargo.toml:24-26` makes tract-gpu and tract-metal unconditional
on macOS. `--rules=only` is parsed inside main, after the OS has loaded and initialized
linked frameworks. Skipping model execution cannot remove that earlier work.

This identifies eager ML/framework linkage as a concrete optimization candidate.
The next comparison should remove that linkage from rules-only startup (a separate
rules core with an optional or dynamically loaded ML backend). This debugger run
does not assign the full measured 3.09 ms pre-main interval to Metal: process
creation, base runtime libraries, other frameworks and scheduling also contribute.

[Debugger transcript](initializers.log), [initial executable-entry inspection](entry.log),
[Cargo dependency tree](metal-dependency.txt), and [machine-readable image list](evidence.json).

Reproduce by building with the command in `evidence.json`, preserving the executable
and its debug-symbol artifacts, and adjusting `initializers.lldb` paths to those
artifacts and the saved one-file workload. Run from the original workload directory:

```sh
rust-lldb --batch -s /absolute/path/to/initializers.lldb
```

The transcript includes pending breakpoints at entry that resolve after the shared
libraries are loaded, actual source/stack frames, the image list, final classification
and normal process exit. ASLR is left enabled. No application code was changed in
this investigation; the PR's release binary was restored after the symbol build.
