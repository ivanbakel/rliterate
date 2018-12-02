# rliterate: a rust-lang implementation of literate

`rliterate` is a implementation in **Rust** of [zyedidia's `literate` programming tool](https://github.com/zyedidia/literate). It aims to be mostly backwards compatible with `literate`, prioritising the most useful features, while adding some extensions. 

That means that not all `.lit` files that worked in `literate` will be guaranteed to work in `rliterate`, but any incompatibilities will be reported with a recommended fix, and any differences in behaviour should be obvious.

## Documentation

For an existing source of not-quite-exact information, please refer to the documentation for `zyedidia/literate`. 

## Binaries

This project produces multiple binaries.

### literate

This is the main binary, and is intended to be used in the same way as the original `literate` tool. Read the command-line help for more information.

### cargo-lit

This is a cargo subcommand for literate **Rust** projects. For single projects, it will try to find the `lit` folder in the project root, and process `.lit` files in that folder, recursing into any subfolders - the resulting code files are placed into the `src` folder. Currently, this also places the documentation files in the `src` folder, but that should be configurable and avoidable soon.

For a workspace, the command acts recursively.

### Differences

 - File-level commands can only be defined once. In practise, this should affect nobody.
 - CSS commands are deprecated, and their behaviour is somewhat different. If you want to add custom CSS stylings to your output, it's recommended you use a specialised tool for making webpages out of the raw markdown or HTML files.
