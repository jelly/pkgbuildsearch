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

Configure the Python index updater by copying the config.cfg.in file to new
location or filename. Configure the git repositories, by default Arch Linux's
community and packages repository is included.

Create or configure the repo the location where the repository's will be cloned
and run the import command:

```
python import-meilisearch.py --config /etc/pkgbuildsearch.cfg
```

Run caddy from the root directory and visit the [local url](http://localhost:8888/).
