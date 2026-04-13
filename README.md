# Voĉa (Vocha)

A Rust toolkit for vocal data processing.

## Sub-projects

### Library: [`textgrid-rs`](./crates/textgrid-rs)

A Rust library for working with Praat TextGrid files, supporting:

- Parsing and stringifying TextGrid files in text format
- Deserializing and serializing TextGrid files in binary format

### GUI tool: [`gridder`](./crates/gridder)

A GUI tool for visualizing and editing TextGrid files, built with egui.

### Library: [`gridder-egui-widgets`](./crates/gridder-egui-widgets)

A Rust library providing reusable egui widgets used by `gridder` that can also
be used independently.
