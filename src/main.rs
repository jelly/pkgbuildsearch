extern crate tantivy;

use std::fs::read_dir;
use std::fs::File;
use std::fs;
use std::io::prelude::*;

use tantivy::schema::*;
use tantivy::Index;


pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}


fn main() -> tantivy::Result<()> {
    let index_path = "/tmp/pkgbuildsearch";
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
    for entry in read_dir("/home/jelle/projects/packages/")? {
        let entry = entry?;
        let path = entry.path();
        let basename = entry.path().clone();
        let pkgbasestr = basename.file_name().unwrap().to_str().unwrap_or("").to_string();

        let pkgbuildfile = format!("{}/trunk/PKGBUILD", &path.display());

        if !path_exists(&pkgbuildfile) {
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
