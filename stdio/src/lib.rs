//! jsonrpc server using stdin/stdout
//!
//! ```no_run
//!
//! use jsonrpc_stdio_server::ServerBuilder;
//! use jsonrpc_core::*;
//!
//! #[tokio::main]
//! async fn main() {
//! 	let mut io = IoHandler::default();
//! 	io.add_method("say_hello", |_params| {
//! 		Ok(Value::String("hello".to_owned()))
//! 	});
//!
//! 	ServerBuilder::new(io).build().await.unwrap();
//! }
//! ```

#![deny(missing_docs)]

use futures03::{compat::*, stream::{StreamExt as _}, sink::SinkExt as _};
use jsonrpc_core::IoHandler;
use log::info;
use std::sync::Arc;

/// Stdio server builder
pub struct ServerBuilder {
	handler: Arc<IoHandler>,
}

impl ServerBuilder {
	/// Returns a new server instance
	pub fn new<T>(handler: T) -> Self
	where
		T: Into<IoHandler>,
	{
		ServerBuilder {
			handler: Arc::new(handler.into()),
		}
	}

	/// Will block until EOF is read or until an error occurs.
	/// The server reads from STDIN line-by-line, one request is taken
	/// per line and each response is written to STDOUT on a new line.
	pub async fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
		use tokio::codec::{FramedRead, FramedWrite, LinesCodec};

		let stdin = tokio::io::stdin();
		let stdout = tokio::io::stdout();

		let mut framed_stdin = FramedRead::new(stdin, LinesCodec::new());
		let mut framed_stdout = FramedWrite::new(stdout, LinesCodec::new());

		while let Some(line) = framed_stdin.next().await.transpose()? {
			let response: String = process_one_request(&self.handler, line).await.unwrap();
			framed_stdout.send(response).await?;
		}

		Ok(())
	}
}

/// Process a request asynchronously
async fn process_one_request(io: &Arc<IoHandler>, input: String) -> Result<String, ()> {
	let response = io.handle_request(&input).compat().await?;

	match response {
		Some(res) => Ok(res),
		None => {
			info!("JSON RPC request produced no response: {:?}", input);
			Ok(String::from(""))
		}
	}
}
