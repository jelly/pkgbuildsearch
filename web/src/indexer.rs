use std::path::PathBuf;
use serde::Deserialize;
use std::collections::HashMap;
use structopt::StructOpt;
use std::fs;
use std::path::Path;
use git2::{Repository, ObjectType, ResetType};
use log::{error, info};
use env_logger::Env;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};


fn parse_path(src: &str) -> Result<PathBuf, &str> {
    let output = PathBuf::from(src);
    if !output.exists() {
        return Err("Path does not exists");
    }
    Ok(output)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "pkgbuildindexer", about, author)]
struct Args {
    /// Meilisearch listen address
    #[structopt(long, default_value = "localhost:7700", env = "PKGBUILDSEARCH_MEILISEARCH_ADDR")]
    meilisearch_listen_address: String,

    /// Meiliearch master key
    #[structopt(long, default_value = "", env = "MEILI_MASTER_KEY", hide_env_values = true)]
    meilisearch_apikey: String,

    /// Config file
    #[structopt(long, default_value = "/etc/pkgbuildsearch.cfg", parse(try_from_str = parse_path))]
    config_file: PathBuf,

    /// Verbose logging
    #[structopt(short)]
    verbose: bool,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    repolocation: String,
    #[serde(rename="repos")]
    repositories: HashMap<String, Repo>,
}

#[derive(Debug, Deserialize)]
struct Repo {
    url: String,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::from_args();

    let buf = fs::read(args.config_file).unwrap();
    let config: ConfigFile = toml::from_slice(&buf).unwrap();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let repolocation = Path::new(&config.repolocation);

    for (reponame, repodata) in config.repositories {
        let repopath = repolocation.join(&reponame);

        // Already cloned, update
        if repopath.exists() {
            let remote_name = "origin";
            let remote_branch = "master";
            info!("Updating repo: {}", reponame);
            let repo = match Repository::open(repopath) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to open: {}", e),
            };

            let head_commit = repo.head().unwrap();
            //let head_object = repo.find_object(head_commit.target().un, None).unwrap();
            let head_tree = head_commit.peel(ObjectType::Tree).unwrap();

            let mut remote = repo.find_remote(remote_name).unwrap();
            remote.fetch(&[remote_branch], None, None).unwrap();
            let stats = remote.stats();
            if stats.local_objects() > 0 {
                info!("Received {}/{} objects", stats.indexed_objects(), stats.total_objects());
            } else {
                info!("Received {}/{} objects", stats.indexed_objects(), stats.total_objects());
            }
            let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
            let fetch_annotated_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();
            let fetch_commit = repo.find_commit(fetch_annotated_commit.id()).unwrap();
            let fetch_object = repo.find_object(fetch_annotated_commit.id(), None).unwrap();

            let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(fetch_commit.time().seconds(), 0), Utc);

            info!("updated to commit: {} on {} UTC", fetch_annotated_commit.id(), dt);

            let fetch_tree = fetch_object.peel(ObjectType::Tree).unwrap();

            //repo.set_head_detached_from_annotated(fetch_commit);
            repo.reset(&fetch_object, ResetType::Hard, None);

            let mut files_changed: Vec<&std::path::Path> = Vec::new();
            let diff = repo.diff_tree_to_workdir(head_tree.as_tree(), None).unwrap();
            for delta in diff.deltas() {
                // TODO: check status, if removed https://docs.rs/git2/0.13.6/git2/enum.Delta.html
                files_changed.push(delta.new_file().path().unwrap());
            }
            dbg!(files_changed);
        } else {
            info!("Initial clone of repo: {}", reponame);
            let repo = match Repository::clone(&repodata.url, repopath) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            };
            info!("cloned {} succesfully", reponame);
        }
    }

    Ok(())
}
