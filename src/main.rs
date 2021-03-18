use std::net::TcpListener;

use newsletter::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tcp_listener = TcpListener::bind("127.0.0.1:8080").expect("tcp error binding to port");
    run(tcp_listener)?.await
}
