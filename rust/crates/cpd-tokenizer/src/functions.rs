//! Function extraction for similarity scoring (issue #999, stage 2).
//!
//! Walks the oxc AST of a JavaScript/TypeScript source and records, for
//! every function declaration, function expression, method and arrow
//! function, its name, span and the pre-order sequence of AST node types
//! inside it. Node *types* only: identifiers and literal values are not
//! part of the sequence, so the summary describes structure.

use crate::line_index::LineIndex;
use cpd_core::models::Location;
use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::GetSpan;

/// A function found in a source, before token ranges are attached.
#[derive(Debug, Clone, PartialEq)]
pub struct RawFunction {
    pub name: String,
    pub start: Location,
    pub end: Location,
    /// Pre-order AST node types of the function, itself included.
    pub kinds: Vec<u16>,
}

/// Formats handled by [`extract_functions`].
pub fn supports_functions(format: &str) -> bool {
    matches!(format, "javascript" | "typescript" | "jsx" | "tsx")
}

/// Extract every function of a JS/TS source. Returns an empty vector for
/// unsupported formats and for sources that fail to parse.
pub fn extract_functions(source: &str, format: &str) -> Vec<RawFunction> {
    if !supports_functions(format) || source.is_empty() {
        return Vec::new();
    }
    let allocator = Allocator::new();
    let source_type = crate::javascript::source_type_for_format(format);
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let line_index = LineIndex::new(source.as_bytes());
    let mut extractor = Extractor {
        frames: Vec::new(),
        out: Vec::new(),
        pending_name: None,
        line_index: &line_index,
        len: source.len(),
    };
    extractor.visit_program(&parsed.program);
    extractor.out
}

struct Frame {
    name: String,
    start: u32,
    end: u32,
    kinds: Vec<u16>,
}

struct Extractor<'i> {
    frames: Vec<Frame>,
    out: Vec<RawFunction>,
    /// Name from the enclosing declarator, property or method, consumed by
    /// the next function node.
    pending_name: Option<String>,
    line_index: &'i LineIndex,
    len: usize,
}

impl Extractor<'_> {
    fn open(&mut self, name: String, start: u32, end: u32) {
        self.frames.push(Frame {
            name,
            start,
            end,
            kinds: Vec::new(),
        });
    }

    fn close(&mut self) {
        let Some(frame) = self.frames.pop() else {
            return;
        };
        let start = (frame.start as usize).min(self.len);
        let end = (frame.end as usize).min(self.len);
        self.out.push(RawFunction {
            name: frame.name,
            start: self.line_index.location(start),
            end: self.line_index.location(end),
            kinds: frame.kinds,
        });
    }
}

impl<'a> Visit<'a> for Extractor<'_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::VariableDeclarator(d) => {
                self.pending_name = d.id.get_identifier_name().map(|n| n.to_string());
            }
            AstKind::MethodDefinition(m) => {
                self.pending_name = m.key.static_name().map(|n| n.into_owned());
            }
            AstKind::PropertyDefinition(p) => {
                self.pending_name = p.key.static_name().map(|n| n.into_owned());
            }
            AstKind::ObjectProperty(p) => {
                self.pending_name = p.key.static_name().map(|n| n.into_owned());
            }
            AstKind::Function(f) => {
                let name =
                    f.id.as_ref()
                        .map(|id| id.name.to_string())
                        .or_else(|| self.pending_name.take())
                        .unwrap_or_else(|| "<anonymous>".to_string());
                let span = f.span;
                self.open(name, span.start, span.end);
            }
            AstKind::ArrowFunctionExpression(a) => {
                let name = self
                    .pending_name
                    .take()
                    .unwrap_or_else(|| "<arrow>".to_string());
                let span = a.span;
                self.open(name, span.start, span.end);
            }
            _ => {}
        }
        let ty = kind.ty() as u16;
        for frame in &mut self.frames {
            frame.kinds.push(ty);
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => self.close(),
            AstKind::VariableDeclarator(_)
            | AstKind::MethodDefinition(_)
            | AstKind::PropertyDefinition(_)
            | AstKind::ObjectProperty(_) => self.pending_name = None,
            _ => {}
        }
        let _ = kind.span();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "export function total(items) {\n  let sum = 0;\n  for (const it of items) { sum += it.price; }\n  return sum;\n}\nconst double = (x) => x * 2;\nclass Cart {\n  add(item) { this.items.push(item); }\n}\nconst obj = { run() { return 1; }, cb: function () { return 2; } };\n";

    #[test]
    fn extracts_declarations_arrows_methods_and_properties_with_names() {
        let fns = extract_functions(SRC, "javascript");
        let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["total", "double", "add", "run", "cb"]);
        let total = &fns[0];
        assert_eq!((total.start.line, total.end.line), (1, 5));
        assert!(total.kinds.len() > 20, "{}", total.kinds.len());
        assert_eq!(total.kinds[0], oxc_ast::AstType::Function as u16);
    }

    #[test]
    fn nested_functions_are_emitted_separately_and_contribute_to_the_outer() {
        let src = "function outer() {\n  const inner = () => 1;\n  return inner();\n}\n";
        let fns = extract_functions(src, "typescript");
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "inner"); // closed first
        assert_eq!(fns[1].name, "outer");
        assert!(fns[1].kinds.len() > fns[0].kinds.len());
    }

    #[test]
    fn unsupported_or_broken_sources_yield_nothing() {
        assert!(extract_functions("def f():\n  pass\n", "python").is_empty());
        assert!(extract_functions("function (", "javascript").is_empty());
        assert!(extract_functions("", "javascript").is_empty());
    }

    #[test]
    fn renamed_copies_share_the_same_kind_sequence() {
        let a = extract_functions("function a(x) { return x + 1; }", "javascript");
        let b = extract_functions("function b(y) { return y + 1; }", "javascript");
        assert_eq!(a[0].kinds, b[0].kinds);
    }
}
