extern crate tantivy;

use std::env;
use std::time::Instant;

use tantivy::query::QueryParser;
use tantivy::Index;

use tantivy::collector::TopDocs;

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

    /*
    // Allows getting with a limit..
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10000))?;
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let foo = retrieved_doc.get_all(pkgbuild);
        println!("{}", &foo[0].text().unwrap());
        //let values = retrieved_doc.get_all();
        //println!("{}", values[0].text);
        println!("{}", schema.to_json(&retrieved_doc));
    }
    */

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
