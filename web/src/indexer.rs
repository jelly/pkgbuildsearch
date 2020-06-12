use std::path::PathBuf;
use serde::Deserialize;
use std::collections::HashMap;
use structopt::StructOpt;
use std::fs;
use std::path::Path;
use git2::Repository;
use log::{error, info};
use env_logger::Env;

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
            info!("Updating repo: {}", reponame);
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
