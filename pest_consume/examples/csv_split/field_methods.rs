//! The other half of the split parser: rule methods for the leaf grammar rules, declared in a
//! separate `impl CSVParser` block from `main.rs`'s `record`/`file` methods.
use merc_pest_consume::match_nodes;

use crate::CSVField;
use crate::CSVParser;
use crate::Node;
use crate::Result;
use crate::Rule;

#[merc_pest_consume::parser_methods]
impl CSVParser {
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    fn number(input: Node) -> Result<f64> {
        input
            .as_str()
            .parse::<f64>()
            // `input.error` links the error to the location in the input file where it occurred.
            .map_err(|e| input.error(e))
    }

    fn string(input: Node<'_>) -> Result<&str> {
        Ok(input.as_str())
    }

    // `pub(crate)` because it's called from `main.rs`'s `record` method.
    pub(crate) fn field(input: Node) -> Result<CSVField> {
        Ok(match_nodes!(input.into_children();
            [number(n)] => CSVField::Number(n),
            [string(s)] => CSVField::String(s),
        ))
    }
}
