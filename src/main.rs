mod index;
mod server;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 8888)]
    port: u16,
    #[arg(short, long, default_value_t =String::from("./ShareFiles"))]
    dir: String,
}

#[tokio::main]
async fn main() {
    let arg = Args::parse();
    server::open_server(arg.port, arg.dir).await.unwrap();
}
