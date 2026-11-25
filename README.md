### A small Git implementation in Rust, built from scratch to understand Git's internals and plumbing commands.

## Implemented

- `init` => Initialize a Git repository
- `hash-object` => Hash file contents as Git blobs and optionally store them
- `cat-file -p` => Read and display blob contents
- `ls-tree` => Parse and list entries from Git tree objects, with support for `--name-only`

## Usage

Initialize a repository:

```bash
cargo run -- init
```

Hash a file without storing the object:

```bash
cargo run -- hash-object README.md
```

Hash and store a file:

```bash
cargo run -- hash-object -w README.md
```

Read a stored blob:

```bash
cargo run -- cat-file -p <object-hash>
```

List entries in a tree object:

```bash
cargo run -- ls-tree <tree-hash>
```

List only filenames:

```bash
cargo run -- ls-tree --name-only <tree-hash>
```

## Learning

This project explores Git's object model, SHA-1 hashing, zlib compression, binary tree parsing, and the filesystem structure behind Git's plumbing commands.

The implementation is written in Rust, with a focus on understanding the underlying mechanisms rather than relying on Git libraries.
