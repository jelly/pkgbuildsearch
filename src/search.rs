extern crate tantivy;

use std::time::Instant;

use tantivy::query::QueryParser;
use tantivy::Index;

use clap::{Arg, App};

use serde_json;


fn main() -> tantivy::Result<()> {
    let matches = App::new("pkgbuildsearch")
                          .version("0.1")
                          .author("Jelle van der Waa <jelle@vdwaa.nl>")
                          .about("Search git repositories")
                          .arg(Arg::with_name("index-path")
                               .help("Index path")
                               .required(true))
                          .arg(Arg::with_name("query")
                               .help("Search query")
                               .required(true))
                          .get_matches();

    let query = matches.value_of("query").unwrap();
    let index_path = matches.value_of("index-path").unwrap();

    // TODO: handle error when index path does not exists or no index
    let directory = std::path::Path::new(&index_path);
    let index = Index::open_in_dir(directory)?;

    let schema = index.schema();
    let pkgbuild = schema.get_field("pkgbuild").unwrap();
    let query_parser = QueryParser::new(schema.clone(), vec![pkgbuild], index.tokenizers().clone());

    let now = Instant::now();
    let query = query_parser.parse_query(query)?;
    let searcher = index.reader()?.searcher();

    let weight = query.weight(&searcher, false)?;
    let schema = index.schema();
    for segment_reader in searcher.segment_readers() {
        let mut scorer = weight.scorer(segment_reader)?;
        let store_reader = segment_reader.get_store_reader();
        while scorer.advance() {
            let doc_id = scorer.doc();
            let doc = store_reader.get(doc_id)?;
            let named_doc = schema.to_named_doc(&doc);
            println!("{}", serde_json::to_string_pretty(&named_doc).unwrap());
        }
    }
    println!("{} ms", now.elapsed().as_millis());

    Ok(())
}
