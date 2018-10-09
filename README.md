# rliterate: a rust-lang implementation of literate

`rliterate` is a implementation in `Rust` of [zydidia's `literate` programming tool](https://github.com/zydidia/literate). It aims to be mostly backwards compatible with `literate`, prioritising the most useful features, while adding some extensions. 

That means that not all `.lit` files that worked in `literate` will be guaranteed to work in `rliterate`, but any incompatibilities will be reported with a recommended fix, and any differences in behaviour should be obvious.

## Documentation

For an existing source of not-quite-exact information, please refer to the documentation for `zydidia/literate`. 

### Differences

 - File-level commands can only be defined once. In practise, this should affect nobody. 
