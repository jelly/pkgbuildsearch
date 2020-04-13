import argparse

from os.path import basename
from glob import glob

import meilisearch


# TODO: Arbitrair
BATCH_SIZE = 50
INDEX_UID = 'pkgbuilds'


def get_index():
    client = meilisearch.Client('http://127.0.0.1:7700')
    indexes = client.get_indexes()
    for index in indexes:
        if index['name'] == INDEX_UID:
            index = client.get_index(INDEX_UID)
            break
    else:
        index = client.create_index(uid=INDEX_UID)

    return index


def update_index(repodir):
    index = get_index()
    docs = []
    count = 0
    # TODO: verify if packages or community
    repo = basename(repodir)

    for entry in glob(f'{repodir}/**/trunk/PKGBUILD'):
        doc = {}
        pkgbase = entry.split('/')[-3]
        doc['pkgbase_id'] = pkgbase
        doc['repo'] = repo

        with open(entry, 'r') as f:
            try:
                doc['body'] = f.read()
            except UnicodeDecodeError as exc:
                print(f'unable to index {pkgbase}, {exc}')
                continue

        docs.append(doc)
        count += 1

        if count == BATCH_SIZE:
            ret = index.add_documents(docs)
            print('adding documents', ret)
            docs = []
            count = 0

    # Add remainder
    if count:
        ret = index.add_documents(docs)


def main(repodir):
    update_index(repodir)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='meilisearch PKGBUILD database indexer')
    parser.add_argument('--repodir', help='location to repository', type=str)
    args = parser.parse_args()

    main(args.repodir)
