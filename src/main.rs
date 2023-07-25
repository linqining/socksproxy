mod server;

fn main() {
    let addr = String::from("127.0.0.1:1080");
    let s = server::Server::new(addr);
    s.run()
}