use i3_lsp::I3Configuration;
use tower_lsp::lsp_types::Diagnostic;

pub fn check_for_duplicate_bindings(cfg: &I3Configuration) -> Vec<Diagnostic> {
    println!("{}", cfg);
    vec![]
}
