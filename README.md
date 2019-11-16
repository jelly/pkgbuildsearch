# PKGBUILD search

A WIP PKGBUILD search REST service, mostly to teach me a Rust with hardcoded
shenanigans.


Requires a clone of Arch Linux's packages, with some fixes:

```
$ git clone --depth 1 https://git.archlinux.org/svntogit/packages.git

# Fix up PKGBUILD's as, the indexer does not handle it at all
$ iconv -f ISO-8859-15 -t utf-8  /home/jelle/projects/pkgbuildsearch/packages/aspell-es/trunk/PKGBUILD -o /home/jelle/projects/pkgbuildsearch/packages/aspell-es/trunk/PKGBUILD
$ iconv -f ISO-8859-15 -t utf-8  /home/jelle/projects/pkgbuildsearch/packages/ntfs-3g/trunk/PKGBUILD -o /home/jelle/projects/pkgbuildsearch/packages/ntfs-3g/trunk/PKGBUILD
```

## Running

```
$ mkdir /tmp/pkgbuildsearch
$ cargo run --bin indexer
$ cargo run --bin search lol
```
