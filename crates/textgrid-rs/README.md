# `textgrid-rs`

[![Crates.io Version](https://img.shields.io/crates/v/textgrid-rs)](https://crates.io/crates/textgrid-rs)
[![docs.rs](https://img.shields.io/docsrs/textgrid-rs)](https://docs.rs/textgrid-rs)
[![GitHub License](https://img.shields.io/github/license/lazurema/vocha)](https://github.com/lazurema/vocha/blob/main/LICENSE)

A crate for working with Praat TextGrid files in Rust.

## Other Rust implementations

- [`textgrid`](https://crates.io/crates/textgrid)
  ([repo link](https://github.com/amirhosseinghanipour/textgrid), for some
  reason it is not linked from the crate page)
  - The
    [README](https://github.com/amirhosseinghanipour/textgrid/blob/707d262c880dcd6205e3c828c50c9cdbbc3a9ece/README.md)
    and the crates.io metadata claim that it is licensed under `CC BY-NC 4.0`,
    but the actual
    [license file](https://github.com/amirhosseinghanipour/textgrid/blob/707d262c880dcd6205e3c828c50c9cdbbc3a9ece/LICENSE)
    appears to be `CC BY-NC-SA 4.0`. License confusion aside, since I'd like to
    process TextGrid files in my MIT-licensed projects, I decided to implement
    my own crate instead of using this one.
- [`textgridde-rs`](https://crates.io/crates/textgridde-rs)
  - It lacks features that I need. At the time of writing, it only supports
    parsing text-format TextGrid files.

## Resources

### Other non-rust implementations in projects with permissive licenses

- **nltk/nltk_contrib** (Python, Apache-2.0):
  [nltk_contrib/textgrid.py](https://github.com/nltk/nltk_contrib/blob/95d1806e2f4e89e960b76a685b1fba2eaa7d5142/nltk_contrib/textgrid.py)

- **vocalpy/crowsetta** (Python, BSD-3-Clause):
  [src/crowsetta/formats/seq/textgrid/parse.py](https://github.com/vocalpy/crowsetta/blob/87c5a4a9eb0b49fd3ca9d789a79998d645bcc271/src/crowsetta/formats/seq/textgrid/parse.py)

- **dopefishh/pympi** (Python, MIT):
  [pympi/Praat.py](https://github.com/dopefishh/pympi/blob/aab19f4c5f84633dfbd5a0fe1d3d84ca9d3397fc/pympi/Praat.py)

- **kylebgorman/textgrid** (Python, MIT):
  [textgrid/textgrid.py](https://github.com/kylebgorman/textgrid/blob/2fba4fd80176018054ce08c9e03e787e5b1840b8/textgrid/textgrid.py)

### Writings

> [!NOTE]
> Some of these writings belong to projects with more restrictive licenses. I
> did not read the source code of those projects.

- <https://www.fon.hum.uva.nl/praat/manual/TextGrid_file_formats.html>
- <https://github.com/vocalpy/crowsetta/issues/242>
