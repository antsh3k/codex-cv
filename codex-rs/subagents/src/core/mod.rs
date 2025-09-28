//! Core subagent implementations for the Codex CLI pipeline.

pub mod spec_parser;
pub mod code_writer;
pub mod tester;
pub mod reviewer;

pub use spec_parser::SpecParserSubagent;
pub use code_writer::CodeWriterSubagent;
pub use tester::TesterSubagent;
pub use reviewer::ReviewerSubagent;