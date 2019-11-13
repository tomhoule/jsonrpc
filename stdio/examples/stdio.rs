use jsonrpc_core::*;
use jsonrpc_stdio_server::ServerBuilder;

#[tokio::main]
async fn main() {
	let mut io = IoHandler::default();
	io.add_method("say_hello", |_params| Ok(Value::String("hello".to_owned())));

	ServerBuilder::new(io).build().await.unwrap();
}
