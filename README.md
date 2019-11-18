# PKGBUILD search

A WIP PKGBUILD search REST service, mostly to teach me a Rust with hardcoded
shenanigans.


Requires a clone of Arch Linux's packages, with some fixes:

```
$ git clone --depth 1 https://git.archlinux.org/svntogit/packages.git

# Fix up PKGBUILD's as, the indexer does not handle it at all, it ignores it. Should be fixed in Git.
$ iconv -f ISO-8859-15 -t utf-8  /home/jelle/projects/pkgbuildsearch/packages/aspell-es/trunk/PKGBUILD -o /home/jelle/projects/pkgbuildsearch/packages/aspell-es/trunk/PKGBUILD
$ iconv -f ISO-8859-15 -t utf-8  /home/jelle/projects/pkgbuildsearch/packages/ntfs-3g/trunk/PKGBUILD -o /home/jelle/projects/pkgbuildsearch/packages/ntfs-3g/trunk/PKGBUILD
```

## Running

```
$ mkdir /tmp/pkgbuildsearch
$ cargo run --bin indexer
$ cargo run --bin search lol
```

## TODO

The design is still work in progress, there are a few things which need to be figured out:

* service architecture, multiple binaries (indexer, rest service) or a single in ram index.
* design a rest API /search?q="msg2"
* add argument parsing lib and --index-path and --repo-path
* figure out updating the index when git changes
* updating git repositories (bare clone or full clone), compare size differences
* handle different encodings such as ISO-8859-15, report them as they should be fixed
* ~~report directories with missing PKGBUILD's~~
* seccomp filtering for webservice and git updating
