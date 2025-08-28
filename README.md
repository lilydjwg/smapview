smapview is a simple program to view processes' different kinds of memory usage on Linux.

This project has been renamed from "swapview" to "smapview", as it can now be used for any data in `/proc/PID/smap_rollup`. It looks at swap usage by default and can be changed, e.g. to view transparent hugepage usage:

```sh
smapview -f AnonHugePages
```

This is the version for daily use. For implementations in different programming languages, see [swapview-rosetta](https://github.com/lilydjwg/swapview-rosetta).

Install:

```sh
cargo install smapview
```

Tips: you can continuously monitor swap usage in a terminal with

```sh
watch -n 1 "smapview | tail -\$((\$LINES - 2)) | cut -b -\$COLUMNS"
```
