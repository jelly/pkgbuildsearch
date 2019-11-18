extern crate tantivy;

use std::env;
use std::time::Instant;

use tantivy::query::QueryParser;
use tantivy::Index;

use serde_json;


fn main() -> tantivy::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let query = &args[1];

    // TODO: handle error when index path does not exists or no index
    let index_path = "/tmp/pkgbuildsearch";
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
            println!("{}", serde_json::to_string(&named_doc).unwrap());
        }
    }
    println!("{} ms", now.elapsed().as_millis());

    Ok(())
}
