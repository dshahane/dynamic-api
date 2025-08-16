// This Rust service demonstrates a dynamic CRUD API where the data model
// is defined by a JSON Schema uploaded by the user. It uses a schemaless
// approach with `serde_json::Value` and validates data at runtime.
//
// This version uses `actix-files` to serve static Swagger UI assets and
// a separate route to serve the OpenAPI specification.

// Import necessary crates.
use actix_web::{http, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use jsonschema::JSONSchema;
use std::collections::HashMap;
use std::sync::{Mutex};
use std::io;

// Import utoipa and utoipa-swagger-ui-dist.
use utoipa::{OpenApi, ToSchema};

// Add the 'actix-files' crate for serving static files.
use actix_files::Files;

// Define a constant for the default port number.
const DEFAULT_PORT: u16 = 7777;

// Define the API documentation using the `OpenApi` macro.
// By deriving `Serialize`, we allow this struct to be converted to JSON.
#[derive(OpenApi, Clone, Serialize)]
#[openapi(
    paths(
        upload_schema,
        create_item,
        get_item,
        update_item,
        delete_item,
    ),
    components(
        schemas(SchemaUpload)
    ),
    info(
        title = "Dynamic CRUD API",
        version = "1.0.0",
        description = "A Rust service that creates dynamic CRUD APIs from user-uploaded JSON schemas."
    )
)]
struct ApiDoc;


// Define a thread-safe, in-memory store for schemas and data.
struct AppState {
    schemas: Mutex<HashMap<String, Value>>,
    data: Mutex<HashMap<String, HashMap<String, Value>>>,
    apidocs: Mutex<utoipa::openapi::OpenApi>
}

// A helper struct for request bodies. The `ToSchema` derive generates
// the OpenAPI schema for this struct automatically. This struct now
// includes a `name` field to dynamically name the schema.
#[derive(Deserialize, Serialize, ToSchema)]
struct SchemaUpload {
    name: String,
    schema: Value,
}

// Handler for the root path to provide a welcoming message and guide the user.
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the Dynamic CRUD API. Please visit /swagger-ui-dist/ to explore the API.")
}


// Handler for uploading a new JSON Schema.
// The `#[utoipa::path]` attribute documents this endpoint.
#[utoipa::path(
    post,
    path = "/api/schema",
    request_body(
        content_type = "application/json",
        example = json!({
            "name": "Task",
            "schema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "completed": { "type": "boolean" }
                },
                "required": ["title", "completed"]
            }
        })
    ),
    responses(
        (status = 200, description = "Schema uploaded successfully")
    )
)]
async fn upload_schema(
    req_body: web::Json<SchemaUpload>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut schemas = data.schemas.lock().unwrap();

    // Use the name provided in the request body to store the schema.
    schemas.insert(req_body.name.clone(), req_body.schema.clone());

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": format!("Schema for '{}' uploaded successfully.", req_body.name)
    }))
}

// Helper function to validate JSON data against a schema.
// This logic remains the same.
fn validate_data(
    schema: &Value,
    instance: &Value,
) -> Result<(), Vec<String>> {
    let compiled_schema = JSONSchema::options()
        .compile(schema)
        .map_err(|e| vec![e.to_string()])?;

    if let Err(errors) = compiled_schema.validate(instance) {
        let error_messages: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
        return Err(error_messages);
    }

    Ok(())
}

// Handler for creating a new item.
#[utoipa::path(
    post,
    path = "/api/{model_name}",
    request_body(
        content_type = "application/json",
        example = json!({
            "title": "Learn Rust",
            "completed": false
        })
    ),
    responses(
        (status = 201, description = "Item created successfully"),
        (status = 400, description = "Validation failed")
    ),
    params(
        ("model_name", description = "The name of the data model")
    )
)]
async fn create_item(
    path: web::Path<String>,
    item: web::Json<Value>,
    data: web::Data<AppState>,
) -> impl Responder {
    let model_name = path.into_inner();
    let schemas = data.schemas.lock().unwrap();

    let schema = match schemas.get(&model_name) {
        Some(s) => s,
        None => return HttpResponse::BadRequest().body(format!("No schema found for model '{}'", model_name)),
    };

    if let Err(errors) = validate_data(schema, &item) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Validation failed",
            "errors": errors
        }));
    }

    let mut item_data = data.data.lock().unwrap();
    let model_data = item_data.entry(model_name.clone()).or_insert_with(HashMap::new);

    let id = format!("{}", uuid::Uuid::new_v4());

    model_data.insert(id.clone(), item.into_inner());

    HttpResponse::Created().json(json!({
        "status": "success",
        "id": id,
        "data": model_data.get(&id)
    }))
}

// Handler for getting a single item.
#[utoipa::path(
    get,
    path = "/api/{model_name}/{id}",
    responses(
        (status = 200, description = "Item found"),
        (status = 404, description = "Item not found")
    ),
    params(
        ("model_name", description = "The name of the data model"),
        ("id", description = "The unique ID of the item")
    )
)]
async fn get_item(
    path: web::Path<(String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (model_name, item_id) = path.into_inner();
    let item_data = data.data.lock().unwrap();

    if let Some(model_data) = item_data.get(&model_name) {
        if let Some(item) = model_data.get(&item_id) {
            return HttpResponse::Ok().json(json!({
                "status": "success",
                "data": item
            }));
        }
    }

    HttpResponse::NotFound().body(format!("Item with ID '{}' not found in model '{}'", item_id, model_name))
}

// Handler for updating an item.
#[utoipa::path(
    put,
    path = "/api/{model_name}/{id}",
    request_body(
        content_type = "application/json",
        example = json!({
            "title": "Master Rust",
            "completed": true
        })
    ),
    responses(
        (status = 200, description = "Item updated successfully"),
        (status = 400, description = "Validation failed"),
        (status = 404, description = "Item not found")
    ),
    params(
        ("model_name", description = "The name of the data model"),
        ("id", description = "The unique ID of the item")
    )
)]
async fn update_item(
    path: web::Path<(String, String)>,
    item: web::Json<Value>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (model_name, item_id) = path.into_inner();
    let schemas = data.schemas.lock().unwrap();

    let schema = match schemas.get(&model_name) {
        Some(s) => s,
        None => return HttpResponse::BadRequest().body(format!("No schema found for model '{}'", model_name)),
    };

    if let Err(errors) = validate_data(schema, &item) {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Validation failed",
            "errors": errors
        }));
    }

    let mut item_data = data.data.lock().unwrap();
    if let Some(model_data) = item_data.get_mut(&model_name) {
        if let Some(existing_item) = model_data.get_mut(&item_id) {
            *existing_item = item.into_inner();
            return HttpResponse::Ok().json(json!({
                "status": "success",
                "message": format!("Item with ID '{}' updated successfully", item_id),
                "data": existing_item
            }));
        }
    }

    HttpResponse::NotFound().body(format!("Item with ID '{}' not found in model '{}'", item_id, model_name))
}

// Handler for deleting an item.
#[utoipa::path(
    delete,
    path = "/api/{model_name}/{id}",
    responses(
        (status = 200, description = "Item deleted successfully"),
        (status = 404, description = "Item not found")
    ),
    params(
        ("model_name", description = "The name of the data model"),
        ("id", description = "The unique ID of the item")
    )
)]
async fn delete_item(
    path: web::Path<(String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (model_name, item_id) = path.into_inner();
    let mut item_data = data.data.lock().unwrap();

    if let Some(model_data) = item_data.get_mut(&model_name) {
        if model_data.remove(&item_id).is_some() {
            return HttpResponse::Ok().json(json!({
                "status": "success",
                "message": format!("Item with ID '{}' deleted successfully", item_id)
            }));
        }
    }

    HttpResponse::NotFound().body(format!("Item with ID '{}' not found in model '{}'", item_id, model_name))
}


// Handler to serve the OpenAPI JSON spec.
async fn serve_openapi_spec(app_state: web::Data<AppState>) -> impl Responder {
    // Return the OpenAPI spec as JSON. The `Serialize` trait on `ApiDoc`
    // makes this possible.
    HttpResponse::Ok().json(app_state.apidocs.lock().unwrap().clone())
}


// Main function to run the web server.
#[actix_web::main]
async fn main() -> io::Result<()> {
    // Create the OpenAPI specification from the `ApiDoc` struct.
    let openapi = ApiDoc::openapi();

    let app_state = web::Data::new(AppState {
        schemas: Mutex::new(HashMap::new()),
        data: Mutex::new(HashMap::new()),
        apidocs: Mutex::new(openapi.clone())
    });

    println!("Service is running at http://127.0.0.1:{}", DEFAULT_PORT);
    println!("Swagger UI available at http://127.0.0.1:{}/swagger-ui/", DEFAULT_PORT);
    println!("OpenAPI spec available at http://127.0.0.1:{}/api-docs/openapi.json", DEFAULT_PORT);

    HttpServer::new(move || {
        let swagger_ui_path = "./static/swagger-ui-dist";
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE])
            .supports_credentials();

        App::new()
            .wrap(cors)
            // Pass the application state to all route handlers.
            .app_data(app_state.clone())
            // Also pass the OpenAPI spec, which is now serializable.
            //.app_data(web::Data::new(openapi.clone()))
            // Serve the static files from a local directory.
            .service(Files::new("/swagger-ui", swagger_ui_path).index_file("index.html"))            // Define a separate route to serve the OpenAPI spec as JSON.
            .service(web::scope("/api-docs")
                .route("/openapi.json", web::get().to(serve_openapi_spec))
            )
            // Define the API routes.
            .service(
                web::scope("/api")
                    .service(web::resource("/schema").route(web::post().to(upload_schema)))
                    .service(web::resource("/{model_name}").route(web::post().to(create_item)))
                    .service(web::resource("/{model_name}/{id}")
                        .route(web::get().to(get_item))
                        .route(web::put().to(update_item))
                        .route(web::delete().to(delete_item))
                    )
            )
            .route("/", web::get().to(index))
    })
        .bind(format!("127.0.0.1:{}", DEFAULT_PORT))?
        .run()
        .await
}
