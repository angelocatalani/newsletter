//! The `newsletter` entry point.

use actix_web::{guard, web, App, HttpRequest, HttpServer, Responder, Route, HttpResponse};
async fn health_check(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

const MAX_PENDING_CONNECTION: u32 = 128;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // HttpServer handles all transport level concerns.
    HttpServer::new(|| {
        // App is where all the application logic lives: routing, middlewares, request
        // handlers, etc.
        App::new()
            // route takes two parameters: path and route
            // path -> template string
            // route -> Route Struct implements the Route trait.
            // The route combines the handler with a set of guards:
            // the check method verifies the guards conditions are met
            // and eventually call the handler
            .route(
                "/",
                Route::new()
                    .guard(guard::Get())
                    .guard(guard::Header("content-type", "text/plain"))
                    .to(greet),
            )
            .route("/{name}", web::get().to(greet))
    })
    .backlog(MAX_PENDING_CONNECTION)
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
