# PKGBUILD search

A simple web frontend to search through Arch Linux's packages PKGBUILD's.

## Dependencies

* meilisearch
* python
* python-meilisearch
* python-pygit2

## Development

For developing a local meilisearch instance is required combined with a caddy
service to handle the proxying to avoid CORS issues.
