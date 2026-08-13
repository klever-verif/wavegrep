#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLayer {
    Parse,
    Semantic,
    Runtime,
}

impl DiagnosticLayer {
    fn as_str(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Semantic => "semantic",
            Self::Runtime => "runtime",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprDiagnostic {
    pub layer: DiagnosticLayer,
    pub code: &'static str,
    pub message: String,
    pub primary_span: Span,
    pub notes: Vec<String>,
}

impl ExprDiagnostic {
    pub fn render(&self, source: &str) -> String {
        let header = format!("{}:{}: {}", self.layer.as_str(), self.code, self.message);
        let location = format!(
            "--> span {}..{}",
            self.primary_span.start, self.primary_span.end
        );
        let excerpt = format!("source: {source}");
        let caret_start = source
            .char_indices()
            .take_while(|(offset, _)| *offset < self.primary_span.start)
            .count();
        let caret_width = source
            .char_indices()
            .filter(|(offset, _)| {
                *offset >= self.primary_span.start && *offset < self.primary_span.end
            })
            .count()
            .max(1);
        let underline = format!(
            "        {}{}",
            " ".repeat(caret_start),
            "^".repeat(caret_width)
        );
        let notes = if self.notes.is_empty() {
            String::new()
        } else {
            self.notes
                .iter()
                .map(|note| format!("note: {note}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        if notes.is_empty() {
            [header, location, excerpt, underline].join("\n")
        } else {
            [header, location, excerpt, underline, notes].join("\n")
        }
    }
}
