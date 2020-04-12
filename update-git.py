from pygit2 import Repository, GIT_RESET_HARD


REPO_LOC = '/tmp/packages'


def update(remote_name='origin', branch='master'):
    repo = Repository(REPO_LOC)

    for remote in repo.remotes:
        if remote.name == remote_name:
            break

    if not remote:
        return

    remote.fetch()
    remote_master_id = repo.lookup_reference(f'refs/remotes/origin/{branch}').target

    head_commit_id = repo.head.target
    repo.reset(remote_master_id, GIT_RESET_HARD)

    diff = repo.diff(head_commit_id, remote_master_id)

    files_changed = []
    for delta in diff.deltas:
        path = delta.new_file.path
        if 'trunk' in path and 'PKGBUILD' in path:
            files_changed.append(delta.new_file.path)

    # Update meilisearch
    print(files_changed)



def main():
    update()


if __name__ == "__main__":
    main()

