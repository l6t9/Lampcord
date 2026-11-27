use cssparser::{
    AtRuleParser, BasicParseErrorKind, CowRcStr, DeclarationParser, ParseError,
    Parser as CssParser, ParserInput, ParserState, QualifiedRuleParser, RuleBodyItemParser,
    RuleBodyParser, StyleSheetParser, Token,
};
use selectors::parser::{ParseRelative, SelectorList, SelectorParseErrorKind};

use super::selector_impl::{WidgetSelectorImpl, WidgetSelectorParser};

#[derive(Clone, Debug)]
pub struct Declaration {
    pub name: String,
    pub raw_value: String,
}

#[derive(Clone)]
pub struct StyleRule {
    pub selectors: SelectorList<WidgetSelectorImpl>,
    pub declarations: Vec<Declaration>,
}

type CssError<'i> = SelectorParseErrorKind<'i>;

pub fn parse_inline_declarations(css: &str) -> Vec<Declaration> {
    parse_stylesheet(&format!("* {{ {css} }}"))
        .into_iter()
        .next()
        .map(|rule| rule.declarations)
        .unwrap_or_default()
}

pub fn parse_stylesheet(css: &str) -> Vec<StyleRule> {
    let mut input = ParserInput::new(css);
    let mut parser = CssParser::new(&mut input);
    let mut top = TopLevelParser;
    StyleSheetParser::new(&mut parser, &mut top)
        .filter_map(Result::ok)
        .flatten()
        .collect()
}

struct TopLevelParser;

impl<'i> QualifiedRuleParser<'i> for TopLevelParser {
    type Prelude = SelectorList<WidgetSelectorImpl>;

    type QualifiedRule = Option<StyleRule>;
    type Error = CssError<'i>;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        SelectorList::parse_disallow_pseudo(&WidgetSelectorParser, input, ParseRelative::No)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut decl_parser = DeclParser;
        let declarations = RuleBodyParser::new(input, &mut decl_parser)
            .filter_map(Result::ok)
            .collect();
        Ok(Some(StyleRule {
            selectors: prelude,
            declarations,
        }))
    }
}

impl<'i> AtRuleParser<'i> for TopLevelParser {
    type Prelude = String;
    type AtRule = Option<StyleRule>;
    type Error = CssError<'i>;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        if name.eq_ignore_ascii_case("keyframes") {
            Ok(input.expect_ident_or_string()?.to_string())
        } else {
            Err(input.new_error(BasicParseErrorKind::AtRuleInvalid(name)))
        }
    }

    fn parse_block<'t>(
        &mut self,
        keyframes_name: Self::Prelude,
        _start: &ParserState,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        let mut frames = Vec::new();
        for block in StyleSheetParser::new(input, &mut KeyframesParser).filter_map(Result::ok) {
            frames.extend(block);
        }
        super::animation::register(&keyframes_name, super::animation::Keyframes { frames });
        Ok(None)
    }
}

struct KeyframesParser;

impl<'i> QualifiedRuleParser<'i> for KeyframesParser {
    type Prelude = Vec<f32>;

    type QualifiedRule = Vec<super::animation::Frame>;
    type Error = CssError<'i>;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        input.parse_comma_separated(|input| {
            let location = input.current_source_location();
            match input.next()?.clone() {
                Token::Ident(ident) if ident.eq_ignore_ascii_case("from") => Ok(0.0),
                Token::Ident(ident) if ident.eq_ignore_ascii_case("to") => Ok(1.0),
                Token::Percentage { unit_value, .. } if (0.0..=1.0).contains(&unit_value) => {
                    Ok(unit_value)
                }
                token => Err(location.new_unexpected_token_error(token)),
            }
        })
    }

    fn parse_block<'t>(
        &mut self,
        offsets: Self::Prelude,
        _start: &ParserState,
        input: &mut CssParser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut decl_parser = DeclParser;
        let declarations: Vec<Declaration> = RuleBodyParser::new(input, &mut decl_parser)
            .filter_map(Result::ok)
            .collect();
        Ok(offsets
            .into_iter()
            .map(|offset| super::animation::Frame {
                offset,
                declarations: declarations.clone(),
            })
            .collect())
    }
}

impl<'i> AtRuleParser<'i> for KeyframesParser {
    type Prelude = ();
    type AtRule = Vec<super::animation::Frame>;
    type Error = CssError<'i>;
}

struct DeclParser;

impl<'i> DeclarationParser<'i> for DeclParser {
    type Declaration = Declaration;
    type Error = CssError<'i>;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut CssParser<'i, 't>,
        _declaration_start: &ParserState,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let value_start = input.state();

        while input.next().is_ok() {}
        let raw_value = input.slice_from(value_start.position()).trim().to_string();
        Ok(Declaration {
            name: name.as_ref().to_ascii_lowercase(),
            raw_value,
        })
    }
}

impl<'i> QualifiedRuleParser<'i> for DeclParser {
    type Prelude = ();
    type QualifiedRule = Declaration;
    type Error = CssError<'i>;
}

impl<'i> AtRuleParser<'i> for DeclParser {
    type Prelude = ();
    type AtRule = Declaration;
    type Error = CssError<'i>;
}

impl<'i> RuleBodyItemParser<'i, Declaration, CssError<'i>> for DeclParser {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        false
    }
}
