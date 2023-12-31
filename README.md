# mdbook-numeq

[![Crates.io](https://img.shields.io/crates/v/mdbook-numeq)](https://crates.io/crates/mdbook-numeq)
[![GitHub License](https://img.shields.io/github/license/yannickseurin/mdbook-numeq)](https://github.com/yannickseurin/mdbook-numeq/blob/main/LICENSE)

An [mdBook](https://github.com/rust-lang/mdBook) preprocessor to automatically number centered equations and later create a link to these equations for a "LaTeX" type experience.

## Installation

Assuming you have mdBook and [mdbook-katex](https://github.com/lzanini/mdbook-katex) installed, install the crate with

```console
$ cargo install mdbook-numeq
```

Then add it as a preprocessor to your `book.toml`:

```toml
[preprocessor.numeq]
```

This crate uses the [`htmlID`](https://katex.org/docs/supported.html#html) command which is disabled by default.
It must enabled by adding `trust = true` to the options of the mdbook-katex preprocessor:

```toml
[preprocessor.katex]
trust = true
```

## Usage

Just add `{{numeq}}` at the end of your centered equation, e.g. (assuming you set the delimiters for centered equations to `\[ ... \]` in the [mdbook-katex preprocessor options](https://github.com/lzanini/mdbook-katex#custom-delimiter))

```text
\[
 a= b {{numeq}}
\]
```

and your equation will be automatically numbered:

> *a = b &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; (1)*


You can optionally provide a label `{{numeq}}{mylabel}`, in which case an anchor will be created.
You can then link to the equation using `{{eqref: mylabel}}`.

## Options

By default, the numbering is per section, meaning the counter is reset to zero at the beginning of each section.
You can choose a global numbering throughout sections by setting the `global` option to true:

```toml
[preprocessor.numeq]
global = true
```

Then, equations will be numbered, say, 1 to 5 in Chapter 1, then 6 to 9 in Chapter 2, etc.

You can choose to add the section number as a prefix to the counter by setting the `prefix` option to true (which makes more sense when `global` is false, but both options are independent).

```toml
[preprocessor.numeq]
prefix = true
```

For example, in Section 3.2, equations will then be numbered 3.2.1, 3.2.2, etc.