mod lsp;
mod parser;

use lsp::I3Backend;
use tower_lsp::{LspService, Server};
use tracing::{Level, event};

#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::never(".", "/tmp/i3-lsp.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    event!(Level::INFO, "Starting lsp server for i3");
    let (service, socket) = LspService::new(|client| I3Backend::new(client));

    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
