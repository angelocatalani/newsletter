//! The `newsletter` entry point.

use std::net::TcpListener;

use actix_web::{
    App,
    guard,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder,
    Route,
    web,
};
use actix_web::dev::Server;
use serde::Deserialize;

const MAX_PENDING_CONNECTION: u32 = 128;

async fn health_check(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[derive(Deserialize)]
struct FormData {
    name: String,
    email: String,
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(tcp_listener: TcpListener) -> std::io::Result<Server> {
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
                "/{name}",
                Route::new()
                    .guard(guard::Get())
                    .guard(guard::Header("content-type", "text/plain"))
                    .to(greet),
            )
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
        .backlog(MAX_PENDING_CONNECTION)
        .listen(tcp_listener)
        .map(HttpServer::run)
}
