extern crate tantivy;

use std::fs::read_dir;
use std::fs::File;
use std::fs;
use std::io::prelude::*;


use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;


pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}


fn main() -> tantivy::Result<()> {
    //let index_path = TempDir::new("tantivy_example_dir")?;
    let index_path = "/tmp/pkgbuildsearch";
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("pkgbase", TEXT | STORED);
    schema_builder.add_text_field("pkgbuild", TEXT);

    let schema = schema_builder.build();
    let index = Index::create_in_dir(&index_path, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?;

    let pkgbase = schema.get_field("pkgbase").unwrap();
    let pkgbuild = schema.get_field("pkgbuild").unwrap();

    /*
    // configuration variable/option
    for entry in read_dir("/home/jelle/projects/pkgbuildsearch/packages/")? {
        let entry = entry?;
        let path = entry.path();
        let basename = entry.path().clone();
        let pkgbasestr = basename.file_name().unwrap().to_str().unwrap_or("").to_string();
        let mut doc = Document::default();
        //println!("Name: {}", path.unwrap().file_name());

        let pkgbuildfile = format!("{}/trunk/PKGBUILD", &path.display());
        //println!("PKGBUILD file: {}", pkgbuildfile);

        //let basedir = path.unwrap().path().clone();
        //let basename = basedir.clone();
        if path_exists(&pkgbuildfile) { 
            //println!("read file: {}", &pkgbuildfile);
            let mut file = File::open(pkgbuildfile)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            //println!("contents: {}", &contents);
            doc.add_text(pkgbase, &pkgbasestr);
            doc.add_text(pkgbuild, &contents);
            
            //let mut file = File::open("{}/trunk/PKGBUILD", path.)?;
            index_writer.add_document(doc);
        }
    }

    index_writer.commit()?;
    */

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![pkgbase, pkgbuild]);

    let query = query_parser.parse_query("fuck")?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        println!("{}", schema.to_json(&retrieved_doc));
    }

    Ok(())
}
