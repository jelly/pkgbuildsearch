extern crate tantivy;

use std::env;

use std::fs::read_dir;
use std::fs::File;
use std::fs;
use std::io::prelude::*;


use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;

use std::time::{Duration, Instant};


use serde_json;


fn main() -> tantivy::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let query = &args[1];

    let index_path = "/tmp/pkgbuildsearch";

    let directory = std::path::Path::new(&index_path);
    let index = Index::open_in_dir(directory)?;
    let schema = index.schema();
    let default_fields: Vec<Field> = schema
        .fields()
        .iter()
        .enumerate()
        .filter(|&(_, ref field_entry)| match *field_entry.field_type() {
            FieldType::Str(ref text_field_options) => {
                text_field_options.get_indexing_options().is_some()
            }
            _ => false,
        })
        .map(|(i, _)| Field(i as u32))
        .collect();
    let query_parser = QueryParser::new(schema.clone(), default_fields, index.tokenizers().clone());

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
    println!("{}", now.elapsed().as_nanos());

    Ok(())
}
