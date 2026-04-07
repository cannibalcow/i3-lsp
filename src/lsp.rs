use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Position, Range, Url,
};
use tower_lsp::{
    Client, LanguageServer,
    lsp_types::{
        CompletionOptions, InitializeParams, InitializeResult, InitializedParams, MessageType,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    },
};
use tracing::{Level, event};

pub struct I3Backend {
    client: Client,
}

impl I3Backend {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl I3Backend {
    async fn check(&self, uri: Url, text: String) {
        let mut diagnosics = Vec::new();

        for (i, line) in text.lines().enumerate() {
            if line.contains("error") {
                diagnosics.push(Diagnostic {
                    range: Range {
                        start: Position::new(i as u32, 0),
                        end: Position::new(i as u32, line.len() as u32),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Sluta the word 'error'".into(),
                    ..Default::default()
                })
            }
        }

        self.client.publish_diagnostics(uri, diagnosics, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for I3Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "i3-lsp initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        event!(Level::INFO, "Did open: {}", params.text_document.text);
        self.check(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        event!(Level::INFO, "Did change: {:?}", params);
        let text = &params.content_changes[0].text;
        self.check(params.text_document.uri, text.clone()).await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        event!(Level::INFO, "Did completion: !! ");
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("bindsym".into(), "Bind a key".into()),
            CompletionItem::new_simple("set".into(), "Set variable".into()),
            CompletionItem::new_simple("exec".into(), "Execute command".into()),
        ])))
    }
}
