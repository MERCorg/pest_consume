//! Same parser as `examples/csv`, but split across two files to demonstrate
//! `#[parser_methods]` + `declare_parser!`: the `Parser` impl is declared once here, while rule
//! methods are implemented in two separate `impl CSVParser` blocks (this file and
//! `field_methods.rs`).
use merc_pest_consume::{Error, Parser, match_nodes};

mod field_methods;

#[allow(dead_code)]
#[derive(Debug)]
enum CSVField<'a> {
    Number(f64),
    String(&'a str),
}
type CSVRecord<'a> = Vec<CSVField<'a>>;
type CSVFile<'a> = Vec<CSVRecord<'a>>;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = merc_pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "../examples/csv/csv.pest"]
pub struct CSVParser;

merc_pest_consume::declare_parser!(parser = CSVParser, rule = Rule);

#[merc_pest_consume::parser_methods]
impl CSVParser {
    fn record(input: Node) -> Result<CSVRecord> {
        Ok(match_nodes!(input.into_children();
            [field(fields)..] => fields.collect(),
        ))
    }

    fn file(input: Node) -> Result<CSVFile> {
        Ok(match_nodes!(input.into_children();
            [record(records).., _] => records.collect(),
        ))
    }
}

fn parse_csv(input_str: &str) -> Result<CSVFile<'_>> {
    // Parse the input into `Nodes`
    let inputs = CSVParser::parse(Rule::file, input_str)?;
    // There should be a single root node in the parsed tree
    let input = inputs.single()?;
    // Consume the `Node` recursively into the final value. `file` (this file) calls into
    // `record` (this file), which calls into `field` (field_methods.rs) via `match_nodes!` --
    // exercising rule-method dispatch across the two impl blocks.
    CSVParser::file(input)
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_csv("-20, 12.5\n42, 0")?;
    let mut sum = 0.;
    for record in parsed {
        for field in record {
            if let CSVField::Number(x) = field {
                sum += x;
            }
        }
    }
    assert_eq!(sum, 34.5);

    let unsuccessful_parse = parse_csv("4, 0.1.1");
    println!("failure: {}", unsuccessful_parse.unwrap_err());

    let successful_parse = parse_csv("-273.15 , ' a string '\n\n42, 0")?;
    println!("success: {:?}", successful_parse);

    Ok(())
}
