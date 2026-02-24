use std::{env, fs};
use utoipa::OpenApi;
use web_server::api::ApiDoc;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage: api_gen <output_file>");
    }

    let output_path = &args[1];
    let openapi = ApiDoc::openapi();

    let content = if output_path.ends_with(".json") {
        openapi.to_json().unwrap()
    } else {
        openapi.to_yaml().unwrap()
    };

    fs::write(output_path, content).expect("Failed to write OpenAPI spec");
    println!("OpenAPI spec successfully generated at: {}", output_path);
}
