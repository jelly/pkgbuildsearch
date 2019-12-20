use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};

use std::time::Instant;

use tantivy::query::QueryParser;
use tantivy::Index;

use tantivy::collector::TopDocs;

#[derive(Debug, Deserialize)]
struct SearchQuery {
    name: Option<String>,
}

#[derive(Serialize)]
struct SearchResults {
    results: Vec<String>,
}

#[get("/search")]
fn search(query: web::Query<SearchQuery>) -> impl Responder {

    let index_path = "/tmp/pkgbuildsearch";
    let directory = std::path::Path::new(&index_path);
    let index = Index::open_in_dir(directory).unwrap();
    let schema = index.schema();
    let pkgbuild = schema.get_field("pkgbuild").unwrap();
    let query_parser = QueryParser::new(schema.clone(), vec![pkgbuild], index.tokenizers().clone());

    let now = Instant::now();
    let index_query = query_parser.parse_query(&query.name.unwrap()).unwrap();
    let searcher = index.reader().unwrap().searcher();


    let top_docs = searcher.search(&index_query, &TopDocs::with_limit(10)).unwrap();

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address).unwrap();
        let foo = retrieved_doc.get_all(pkgbuild);
        println!("{}", &foo[0].text().unwrap());
        //let values = retrieved_doc.get_all();
        //println!("{}", values[0].text);
        println!("{}", schema.to_json(&retrieved_doc));
    }

    /*
    let results = match &query.name {
        Some(name) => vec![name.clone()],
        None => vec![],
    };
    */

    HttpResponse::Ok().json(SearchResults {
        results: results,
    })
}

fn main() {
    HttpServer::new(|| {
        App::new()
            .service(search)
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
