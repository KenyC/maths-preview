use std::collections::HashSet;

use rex::parser::{nodes::{Accent, Array, AtomChange, ColSeparator, ExtendedDelimiter, GenFraction, PlainText, Radical, Scripts, Stack}, symbols::Symbol, ParseNode};



pub(crate) fn collect_chars(node : &ParseNode, characters : &mut HashSet<char>) {
    match node {
        ParseNode::Symbol(s) => extract_codepoint(characters, s),
        ParseNode::Delimited(delimited) => {
            for symbol in delimited.delimiters() {
                extract_codepoint(characters, symbol);
            }
            for nodes in delimited.inners() {
                for node in nodes {
                    collect_chars(node, characters)
                }
            }
        },
        ParseNode::ExtendedDelimiter(ExtendedDelimiter { symbol, .. }) => 
            extract_codepoint(characters, symbol),
        ParseNode::Radical(Radical { 
            inner, 
            character 
        }) => {
            characters.insert(*character);
            inner.into_iter().for_each(|node| { collect_chars(node, characters); });
        },
        ParseNode::GenFraction(GenFraction {
            numerator,
            denominator,
            left_delimiter,
            right_delimiter,
            ..
        }) => {
            left_delimiter.map(|s| extract_codepoint(characters, &s)).unwrap_or_default();
            right_delimiter.map(|s| extract_codepoint(characters, &s)).unwrap_or_default();
            Iterator::chain(numerator.iter(), denominator.iter()).for_each(|node| { collect_chars(node, characters); });
        },
        ParseNode::Scripts(Scripts {
            base,
            superscript,
            subscript,
        }) 
        => {
            let nodes = Iterator::chain(
                base.iter().map(Box::as_ref), 
                Iterator::chain(
                    superscript.iter().flatten(),
                    subscript.iter().flatten(),
                )
            );
            nodes.for_each(|node| collect_chars(node, characters));
        },
        ParseNode::Accent(Accent { symbol, nucleus, .. }) 
        => {
            extract_codepoint(characters, symbol);
            nucleus.iter().for_each(|node| collect_chars(node, characters));
        },
        ParseNode::PlainText(PlainText { text }) 
        => {
            for character in text.chars() {
                characters.insert(character);
            }
        },
          ParseNode::AtomChange(AtomChange { inner, .. }) 
        | ParseNode::Color(rex::parser::nodes::Color { inner, .. }) 
        | ParseNode::Group(inner) 
        => {
            inner.iter().for_each(|node| collect_chars(node, characters));
        },
        ParseNode::Stack(Stack { lines, .. }) 
        => {
            lines.iter().flatten().for_each(|node| collect_chars(node, characters));
        },
        ParseNode::Array(Array { col_format, rows, left_delimiter, right_delimiter, .. }) 
        => {
            left_delimiter.map(|s| extract_codepoint(characters, &s)).unwrap_or_default();
            right_delimiter.map(|s| extract_codepoint(characters, &s)).unwrap_or_default();
            rows.iter().flatten().flatten().for_each(|node| collect_chars(node, characters));

            for col_sep in col_format.separators.iter().flatten() {
                if let ColSeparator::AtExpression(seps) = col_sep {
                    seps.iter().for_each(|node| collect_chars(node, characters));
                }
            }
        },
          ParseNode::DummyNode(_) 
        | ParseNode::Rule(_) 
        | ParseNode::Kerning(_) 
        | ParseNode::Style(_) 
        => {

        },
    }
}

pub(crate) fn extract_codepoint(characters: &mut HashSet<char>, symbol: &Symbol) {
    characters.insert(symbol.codepoint);
}
