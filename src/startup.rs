use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{
    guard,
    web,
    App,
    HttpRequest,
    HttpServer,
    Responder,
    Route,
};
use sqlx::PgPool;

use crate::routes::*;
use tracing_actix_web::TracingLogger;

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

pub fn run(
    tcp_listener: TcpListener,
    postgres_pool: PgPool,
    max_pending_connections: u32,
) -> std::io::Result<Server> {
    let web_data_pool = web::Data::new(postgres_pool);

    // HttpServer handles all transport level concerns.
    HttpServer::new(move || {
        // App is where all the application logic lives: routing, middlewares, request
        // handlers, etc.
        App::new()
            .wrap(TracingLogger)
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
            // we need to clone the input connection  because the current closure will be called
            // multiple times (in fact it is of type Fn not FnOnce) and the input connection
            // would not be available anymore at the next call otherwise.
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(web_data_pool.clone())
    })
    .backlog(max_pending_connections)
    .listen(tcp_listener)
    .map(HttpServer::run)
}
