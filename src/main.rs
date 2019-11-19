extern crate clap;
extern crate tantivy;

use std::fs::read_dir;
use std::fs::File;
use std::fs;
use std::io::prelude::*;

use clap::{Arg, App};

use tantivy::schema::*;
use tantivy::Index;


fn indexer(repo_path: &str, index_path: &str) -> tantivy::Result<()> {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("pkgbase", TEXT | STORED);
    schema_builder.add_text_field("pkgbuild", TEXT | STORED);

    let schema = schema_builder.build();
    let index = Index::create_in_dir(&index_path, schema.clone())?;

    // 50 MiB
    let mut index_writer = index.writer(50_000_000)?;

    let pkgbase = schema.get_field("pkgbase").unwrap();
    let pkgbuild = schema.get_field("pkgbuild").unwrap();

    // configuration variable/option
    for entry in read_dir(repo_path)? {
        let entry = entry?;
        let path = entry.path();
        let basename = entry.path().clone();
        let pkgbasestr = basename.file_name().unwrap().to_str().unwrap_or("").to_string();

        let pkgbuildfile = format!("{}/trunk/PKGBUILD", &path.display());

        if fs::metadata(&pkgbuildfile).is_err() {
            println!("PKGBUILD not found: {}", pkgbuildfile);
            continue;
        }

        println!("indexing {}", &pkgbuildfile);
        let file = File::open(pkgbuildfile);
        if file.is_err() {
            println!("unable to open file: {:?}", file.unwrap_err());
            continue;
        }

        let mut contents = String::new();
        let res = file.unwrap().read_to_string(&mut contents);
        if res.is_err() {
            println!("unable to read file: {:?}", res.unwrap_err());
            continue;
        }

        let mut doc = Document::default();
        doc.add_text(pkgbase, &pkgbasestr);
        doc.add_text(pkgbuild, &contents);

        index_writer.add_document(doc);
    }

    index_writer.commit()?;

    println!("Finished indexing");

    Ok(())
}


fn main() -> tantivy::Result<()> {
    let matches = App::new("pkgbuildindexer")
                          .version("0.1")
                          .author("Jelle van der Waa <jelle@vdwaa.nl>")
                          .about("Index git repositories")
                          .arg(Arg::with_name("repo-path")
                               .help("Git repository path")
                               .required(true))
                          .arg(Arg::with_name("index-path")
                               .help("Index path")
                               .required(true))
                          .get_matches();

    let repo_path = matches.value_of("repo-path").unwrap();
    let index_path = matches.value_of("index-path").unwrap();
    indexer(repo_path, index_path)
}
