import argparse
import os
import logging

from configparser import ConfigParser
from glob import glob

import meilisearch

import pygit2
#from pygit2 import Repository, GIT_RESET_HARD


# TODO: Arbitrair batch size
BATCH_SIZE = 50
INDEX_UID = 'pkgbuilds'
REPODIR = '/var/lib/pkgbuildsearch'
CONFIG = '/etc/pkgbuildsearch.cfg'


def get_index():
    # TODO: configuration
    client = meilisearch.Client('http://127.0.0.1:7700')
    indexes = client.get_indexes()
    for index in indexes:
        if index['name'] == INDEX_UID:
            index = client.get_index(INDEX_UID)
            break
    else:
        index = client.create_index(uid=INDEX_UID)

    return index


def update_index(repo, changes):
    index = get_index()
    deletes = []
    docs = []
    count = 0
    repodir = repo['dir']
    repo = os.path.basename(repodir)

    if not changes:
        changes = glob(f'{repodir}/**/trunk/PKGBUILD')

    for entry in changes:
        doc = {}
        pkgbase = entry.split('/')[-3]
        doc['pkgbase_id'] = pkgbase
        doc['repo'] = repo

        # PKGBUILD might have been removed
        if not os.path.exists(entry):
            deletes.append(pkgbase)
            continue

        with open(entry, 'r') as f:
            try:
                doc['body'] = f.read()
            except UnicodeDecodeError as exc:
                logging.warning('unable to index %s, %s', pkgbase, exc)
                continue

        docs.append(doc)
        count += 1

        if count == BATCH_SIZE:
            ret = index.add_documents(docs)
            logging.info("adding documents: '%s'", ret)
            docs = []
            count = 0

    # Add remainder
    if count:
        ret = index.add_documents(docs)

    if deletes:
        index.delete_documents(deletes)
        logging.info('deleted %s documents', len(deletes))


def update_repo(repodir, remote_name='origin', branch='master'):
    repo = pygit2.Repository(repodir)

    for remote in repo.remotes:
        if remote.name == remote_name:
            break

    if not remote:
        logging.error("no remote found, unable to update")
        return []

    remote.fetch()
    remote_master_id = repo.lookup_reference(f'refs/remotes/origin/{branch}').target

    head_commit_id = repo.head.target
    repo.reset(remote_master_id, pygit2.GIT_RESET_HARD)

    diff = repo.diff(head_commit_id, remote_master_id)

    files_changed = []
    for delta in diff.deltas:
        path = delta.new_file.path
        if 'trunk' in path and 'PKGBUILD' in path:
            files_changed.append(delta.new_file.path)

    logging.info("updated repo: '%s' with %s changes", os.path.basename(repodir), len(files_changed))

    # Append clone directory for indexing
    files_changed = (f'{repodir}/{change}' for change in files_changed)

    return files_changed


def update_repos(repos):
    changes = []

    for repo in repos:
        repodir = repo['dir']
        # Already cloned, update
        if os.path.exists(repodir):
            logging.info("Updating repo: '%s'", os.path.basename(repodir))
            changes = update_repo(repodir)
        else:
            logging.info("Initial clone of repo: '%s'", os.path.basename(repodir))
            pygit2.clone_repository(repo['url'], repodir)

    return changes


def init_logging(log_level):
    numeric_level = getattr(logging, log_level.upper(), None)
    if not isinstance(numeric_level, int):
        raise ValueError('Invalid log level: %s' % log_level)

    fmt = '%(asctime)s -> %(levelname)s: %(message)s'
    handler = logging.StreamHandler()
    handler.setFormatter(logging.Formatter(fmt))

    root = logging.getLogger()
    root.setLevel(numeric_level)
    root.addHandler(handler)


def parse_config(configfile):
    repos = []

    config = ConfigParser()
    config.read(configfile)

    general = config['general']
    repodir = general.get('repo-location', REPODIR)

    # All sections apart from [general] are considered Git repos
    for section in config.sections():
        url = config[section].get('url')
        if section == "general":
            continue

        if not url:
            logging.warning('section "%s" as no url', section)
            continue

        repos.append({
            'dir': os.path.join(repodir, section),
            'name': section,
            'url': url,
        })

    return repos


def main(configfile, log_level):
    init_logging(log_level)
    repos = parse_config(configfile)
    
    #changes = update_repos(repos)

    for repo in repos:
        changes = []
        repodir = repo['dir']

        # Already cloned, update
        if os.path.exists(repodir):
            logging.info("Updating repo: '%s'", os.path.basename(repodir))
            changes = update_repo(repodir)
        else:
            logging.info("Initial clone of repo: '%s'", os.path.basename(repodir))
            pygit2.clone_repository(repo['url'], repodir)

        update_index(repo, changes)


def is_file(filepath):
    if not os.path.isfile(filepath):
        raise argparse.ArgumentTypeError(f'is_file: {filepath} is not a valid file')
    if not os.access(filepath, os.R_OK):
        raise argparse.ArgumentTypeError(f'is_file: {filepath} is not a readable file')
    return filepath


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='PKGBUILDSearch importer')
    parser.add_argument('--config', default=CONFIG, type=is_file,
                        help='Specify an alternative configuration file to read')
    parser.add_argument('--log-level', default='INFO', help='log level (default INFO)')
    args = parser.parse_args()
    main(args.config, args.log_level)
