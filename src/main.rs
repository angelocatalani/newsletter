use newsletter::run;
use socket2::{
    Domain,
    Socket,
    Type,
};
use std::net::{
    SocketAddr,
    TcpListener,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create a TCP listener bound to two addresses.
    // let socket = Socket::new(Domain::IPV6, Type::STREAM, None)?;
    // todo: solve cargo audit warning
    // let address: SocketAddr = "[::1]:12345".parse().unwrap();
    // socket.bind(&address.into())?;
    // socket.set_only_v6(false)?;
    // socket.listen(128)?;

    // let listener: TcpListener = socket.into();

    let tcp_listener = TcpListener::bind("127.0.0.1:8080").expect("tcp error binding to port");
    run(tcp_listener)?.await
}
