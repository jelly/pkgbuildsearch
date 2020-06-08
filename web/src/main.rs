use std::collections::HashMap;
use std::path::PathBuf;

use actix_http::{body::Body, Response};
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer, Result};
use tera::Tera;
use meilisearch_sdk::{document::Document, client::Client, search::Query};
use serde::{Serialize, Deserialize};
use structopt::StructOpt;

fn parse_path(src: &str) -> Result<PathBuf, &str> {
    let output = PathBuf::from(src);
    if !output.exists() {
        return Err("Path does not exists");
    }
    Ok(output)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "pkgbuildsearchweb", about, author)]
struct Args {
    /// Host address to bind to.  Accepts hostnames.
    #[structopt(long, default_value = "localhost:8080", env = "PKGBUILDSEARCH_ADDR")]
    bind_addr: String,

    /// Base template directory, default $CARGO_MANIFEST_DIR/**/*
    #[structopt(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/templates"), env = "PKGBUILDSEARCH_TEMPLATE", parse(try_from_str = parse_path))]
    template_dir: PathBuf,

    /// Base template directory, default $CARGO_MANIFEST_DIR/**/*
    #[structopt(long, default_value = "localhost:7700", env = "PKGBUILDSEARCH_MEILISEARCH_ADDR")]
    meilisearch_addr: String,

    /// Base template directory, default $CARGO_MANIFEST_DIR/**/*
    #[structopt(long, default_value = "localhost:7700", env = "PKGBUILDSEARCH_MEILISEARCH_APIKEY", hide_env_values = true)]
    meilisearch_apikey: String,
}


#[derive(Serialize, Deserialize, Debug)]
struct Formatted {
    pkgbase_id: String,
    body: String,
    repo: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Pkgbuild {
    pkgbase_id: String,
    body: String,
    repo: String,
    _formatted: Formatted,
}

#[derive(Serialize, Deserialize, Debug)]
struct ParsedResult {
    pkgbase_id: String,
    repo: String,
    parts: Vec<String>,
}

struct AppData<'a> {
    tera: tera::Tera,
    client: meilisearch_sdk::client::Client<'a>
}

// That trait is required to make a struct usable by an index
impl Document for Pkgbuild {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.pkgbase_id
    }
}

fn format_hits(hits: &Vec<Pkgbuild>, ) -> Vec<ParsedResult> {
    let mut formatted_hits = Vec::new();

    for hit in hits {
        let mut parts = Vec::new();
        let lines: Vec<&str> = hit._formatted.body.split('\n').collect();
        let mut outside = 0;

        for (i, line) in lines.iter().enumerate() {
            let matches = line.find("<em>");

            match matches {
                Some(_) => {
                    // TODO: move logic to a seperate function.
                    if outside > 0 {
                        outside = outside - 1;
                        continue
                    }

                    let mut lower = 0;
                    if i > 2 {
                        lower = i - 2;
                    }

                    let mut upper = lines.len();
                    if i + 2 < upper { 
                        upper = i + 2;
                    }

                    let part = lines.get(lower..upper).unwrap();
                    parts.push(part.join("<br>"));
                    
                    // Skip next two lines
                    outside = 2;
                },
                None => continue,
            }
        }


        let pkgbuild = ParsedResult  {
            pkgbase_id: hit.pkgbase_id.clone(),
            repo: hit.repo.clone(),
            parts: parts
        };

        formatted_hits.push(pkgbuild)
    }

    formatted_hits
}


// store tera template in application state
async fn index(
    tmpl: web::Data<tera::Tera>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let default_query = &String::from("");
    let name = query.get("q").unwrap_or(default_query);
    let mut ctx = tera::Context::new();
    ctx.insert("query", &name.to_owned());

        // submitted form
    if ! name.is_empty() {
        // TODO: pass index to this function and create client in main, does the client handle reconnects?
        let client = Client::new("http://localhost:7700", "");
        match client.get_index("pkgbuilds") {
            Ok(pkgbuilds) => {
            let mquery = Query::new(&name).with_limit(25).with_attributes_to_highlight("*");
            let searchresult = pkgbuilds.search::<Pkgbuild>(&mquery).unwrap();
            let hits = searchresult.hits;

            let formatted_hits = format_hits(&hits);

            ctx.insert("hits", &hits);
            ctx.insert("formatted_hits", &formatted_hits);
            ctx.insert("processing_time_ms", &searchresult.processing_time_ms);
            ctx.insert("nb_hits", &searchresult.nb_hits);
            },
            Err(_error) => {
                // TODO: log error type with switch?
                // Add eroror handling to template.
                // TODO: show error?
                ctx.insert("error", "yes");
            }
        }
    }

    let s = tmpl.render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok()
       .content_type("text/html")
       .header("X-Content-Type-Options", "nosniff")
       .header("X-Frame-Options", "SAMEORIGIN")
       .header("X-XSS-Protection", "1; mode=block")
       .header("Content-Security-Policy", "default-src 'none'; style-src 'unsafe-inline'")
       .header("Referrer-Policy", "no-referrer")
       .body(s))
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> Response<Body> {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        Response::build(res.status())
            .content_type("text/plain")
            .body(e.to_string())
    };

    let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("error.html", &context);

            match body {
                Ok(body) => Response::build(res.status())
                    .content_type("text/html")
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // TODO: set log level via structopt
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let args = Args::from_args();
    let mut template_dir = args.template_dir;
    template_dir.push("**/*");

    let meilisearch_addr = format!("https://{}", args.meilisearch_addr);
    let client = Client::new(meilisearch_addr.as_str(), args.meilisearch_apikey.as_str());

    HttpServer::new(move || {
        let tera =
            Tera::new(template_dir.to_str().unwrap()).unwrap();

        App::new()
            .data(tera)
            .wrap(middleware::Logger::default()) // enable logger
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::scope("").wrap(error_handlers()))
    })
    .bind(args.bind_addr)?
    .run()
    .await
}
