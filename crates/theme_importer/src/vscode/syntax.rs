use indexmap::IndexMap;
use serde::Deserialize;
use strum::EnumIter;

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum VsCodeTokenScope {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub struct VsCodeTokenColor {
    pub name: Option<String>,
    pub scope: Option<VsCodeTokenScope>,
    pub settings: VsCodeTokenColorSettings,
}

#[derive(Debug, Deserialize)]
pub struct VsCodeTokenColorSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    #[serde(rename = "fontStyle")]
    pub font_style: Option<String>,
}

#[derive(Debug, PartialEq, Copy, Clone, EnumIter)]
pub enum GramSyntaxToken {
    Attribute,
    Boolean,
    Comment,
    CommentDoc,
    Constant,
    Constructor,
    Embedded,
    Emphasis,
    EmphasisStrong,
    Enum,
    Function,
    Hint,
    Keyword,
    Label,
    LinkText,
    LinkUri,
    Number,
    Operator,
    Predictive,
    Preproc,
    Primary,
    Property,
    Punctuation,
    PunctuationBracket,
    PunctuationDelimiter,
    PunctuationListMarker,
    PunctuationSpecial,
    String,
    StringEscape,
    StringRegex,
    StringSpecial,
    StringSpecialSymbol,
    Tag,
    TextLiteral,
    Title,
    Type,
    Variable,
    VariableSpecial,
    Variant,
}

impl std::fmt::Display for GramSyntaxToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GramSyntaxToken::Attribute => "attribute",
                GramSyntaxToken::Boolean => "boolean",
                GramSyntaxToken::Comment => "comment",
                GramSyntaxToken::CommentDoc => "comment.doc",
                GramSyntaxToken::Constant => "constant",
                GramSyntaxToken::Constructor => "constructor",
                GramSyntaxToken::Embedded => "embedded",
                GramSyntaxToken::Emphasis => "emphasis",
                GramSyntaxToken::EmphasisStrong => "emphasis.strong",
                GramSyntaxToken::Enum => "enum",
                GramSyntaxToken::Function => "function",
                GramSyntaxToken::Hint => "hint",
                GramSyntaxToken::Keyword => "keyword",
                GramSyntaxToken::Label => "label",
                GramSyntaxToken::LinkText => "link_text",
                GramSyntaxToken::LinkUri => "link_uri",
                GramSyntaxToken::Number => "number",
                GramSyntaxToken::Operator => "operator",
                GramSyntaxToken::Predictive => "predictive",
                GramSyntaxToken::Preproc => "preproc",
                GramSyntaxToken::Primary => "primary",
                GramSyntaxToken::Property => "property",
                GramSyntaxToken::Punctuation => "punctuation",
                GramSyntaxToken::PunctuationBracket => "punctuation.bracket",
                GramSyntaxToken::PunctuationDelimiter => "punctuation.delimiter",
                GramSyntaxToken::PunctuationListMarker => "punctuation.list_marker",
                GramSyntaxToken::PunctuationSpecial => "punctuation.special",
                GramSyntaxToken::String => "string",
                GramSyntaxToken::StringEscape => "string.escape",
                GramSyntaxToken::StringRegex => "string.regex",
                GramSyntaxToken::StringSpecial => "string.special",
                GramSyntaxToken::StringSpecialSymbol => "string.special.symbol",
                GramSyntaxToken::Tag => "tag",
                GramSyntaxToken::TextLiteral => "text.literal",
                GramSyntaxToken::Title => "title",
                GramSyntaxToken::Type => "type",
                GramSyntaxToken::Variable => "variable",
                GramSyntaxToken::VariableSpecial => "variable.special",
                GramSyntaxToken::Variant => "variant",
            }
        )
    }
}

impl GramSyntaxToken {
    pub fn find_best_token_color_match<'a>(
        &self,
        token_colors: &'a [VsCodeTokenColor],
    ) -> Option<&'a VsCodeTokenColor> {
        let mut ranked_matches = IndexMap::new();

        for (ix, token_color) in token_colors.iter().enumerate() {
            if token_color.settings.foreground.is_none() {
                continue;
            }

            let Some(rank) = self.rank_match(token_color) else {
                continue;
            };

            if rank > 0 {
                ranked_matches.insert(ix, rank);
            }
        }

        ranked_matches
            .into_iter()
            .max_by_key(|(_, rank)| *rank)
            .map(|(ix, _)| &token_colors[ix])
    }

    fn rank_match(&self, token_color: &VsCodeTokenColor) -> Option<u32> {
        let candidate_scopes = match token_color.scope.as_ref()? {
            VsCodeTokenScope::One(scope) => vec![scope],
            VsCodeTokenScope::Many(scopes) => scopes.iter().collect(),
        }
        .iter()
        .flat_map(|scope| scope.split(',').map(|s| s.trim()))
        .collect::<Vec<_>>();

        let scopes_to_match = self.to_vscode();
        let number_of_scopes_to_match = scopes_to_match.len();

        let mut matches = 0;

        for (ix, scope) in scopes_to_match.into_iter().enumerate() {
            // Assign each entry a weight that is inversely proportional to its
            // position in the list.
            //
            // Entries towards the front are weighted higher than those towards the end.
            let weight = (number_of_scopes_to_match - ix) as u32;

            if candidate_scopes.contains(&scope) {
                matches += 1 + weight;
            }
        }

        Some(matches)
    }

    pub fn fallbacks(&self) -> &[Self] {
        match self {
            GramSyntaxToken::CommentDoc => &[GramSyntaxToken::Comment],
            GramSyntaxToken::Number => &[GramSyntaxToken::Constant],
            GramSyntaxToken::VariableSpecial => &[GramSyntaxToken::Variable],
            GramSyntaxToken::PunctuationBracket
            | GramSyntaxToken::PunctuationDelimiter
            | GramSyntaxToken::PunctuationListMarker
            | GramSyntaxToken::PunctuationSpecial => &[GramSyntaxToken::Punctuation],
            GramSyntaxToken::StringEscape
            | GramSyntaxToken::StringRegex
            | GramSyntaxToken::StringSpecial
            | GramSyntaxToken::StringSpecialSymbol => &[GramSyntaxToken::String],
            _ => &[],
        }
    }

    fn to_vscode(self) -> Vec<&'static str> {
        match self {
            GramSyntaxToken::Attribute => vec!["entity.other.attribute-name"],
            GramSyntaxToken::Boolean => vec!["constant.language"],
            GramSyntaxToken::Comment => vec!["comment"],
            GramSyntaxToken::CommentDoc => vec!["comment.block.documentation"],
            GramSyntaxToken::Constant => vec!["constant", "constant.language", "constant.character"],
            GramSyntaxToken::Constructor => {
                vec!["entity.name.tag", "entity.name.function.definition.special.constructor"]
            }
            GramSyntaxToken::Embedded => vec!["meta.embedded"],
            GramSyntaxToken::Emphasis => vec!["markup.italic"],
            GramSyntaxToken::EmphasisStrong => {
                vec!["markup.bold", "markup.italic markup.bold", "markup.bold markup.italic"]
            }
            GramSyntaxToken::Enum => vec!["support.type.enum"],
            GramSyntaxToken::Function => vec!["entity.function", "entity.name.function", "variable.function"],
            GramSyntaxToken::Hint => vec![],
            GramSyntaxToken::Keyword => vec![
                "keyword",
                "keyword.other.fn.rust",
                "keyword.control",
                "keyword.control.fun",
                "keyword.control.class",
                "punctuation.accessor",
                "entity.name.tag",
            ],
            GramSyntaxToken::Label => vec!["label", "entity.name", "entity.name.import", "entity.name.package"],
            GramSyntaxToken::LinkText => vec!["markup.underline.link", "string.other.link"],
            GramSyntaxToken::LinkUri => vec!["markup.underline.link", "string.other.link"],
            GramSyntaxToken::Number => vec!["constant.numeric", "number"],
            GramSyntaxToken::Operator => vec!["operator", "keyword.operator"],
            GramSyntaxToken::Predictive => vec![],
            GramSyntaxToken::Preproc => vec!["preproc", "meta.preprocessor", "punctuation.definition.preprocessor"],
            GramSyntaxToken::Primary => vec![],
            GramSyntaxToken::Property => vec![
                "variable.member",
                "support.type.property-name",
                "variable.object.property",
                "variable.other.field",
            ],
            GramSyntaxToken::Punctuation => vec![
                "punctuation",
                "punctuation.section",
                "punctuation.accessor",
                "punctuation.separator",
                "punctuation.definition.tag",
            ],
            GramSyntaxToken::PunctuationBracket => vec![
                "punctuation.bracket",
                "punctuation.definition.tag.begin",
                "punctuation.definition.tag.end",
            ],
            GramSyntaxToken::PunctuationDelimiter => vec![
                "punctuation.delimiter",
                "punctuation.separator",
                "punctuation.terminator",
            ],
            GramSyntaxToken::PunctuationListMarker => {
                vec!["markup.list punctuation.definition.list.begin"]
            }
            GramSyntaxToken::PunctuationSpecial => vec!["punctuation.special"],
            GramSyntaxToken::String => vec!["string"],
            GramSyntaxToken::StringEscape => {
                vec!["string.escape", "constant.character", "constant.other"]
            }
            GramSyntaxToken::StringRegex => vec!["string.regex"],
            GramSyntaxToken::StringSpecial => vec!["string.special", "constant.other.symbol"],
            GramSyntaxToken::StringSpecialSymbol => {
                vec!["string.special.symbol", "constant.other.symbol"]
            }
            GramSyntaxToken::Tag => vec!["tag", "entity.name.tag", "meta.tag.sgml"],
            GramSyntaxToken::TextLiteral => vec!["text.literal", "string"],
            GramSyntaxToken::Title => vec!["title", "entity.name"],
            GramSyntaxToken::Type => vec![
                "entity.name.type",
                "entity.name.type.primitive",
                "entity.name.type.numeric",
                "keyword.type",
                "support.type",
                "support.type.primitive",
                "support.class",
            ],
            GramSyntaxToken::Variable => vec![
                "variable",
                "variable.language",
                "variable.member",
                "variable.parameter",
                "variable.parameter.function-call",
            ],
            GramSyntaxToken::VariableSpecial => vec![
                "variable.special",
                "variable.member",
                "variable.annotation",
                "variable.language",
            ],
            GramSyntaxToken::Variant => vec!["variant"],
        }
    }
}
