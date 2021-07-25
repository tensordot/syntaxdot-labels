use std::io::BufWriter;

use anyhow::{Context, Result};
use clap::{App, AppSettings, Arg, ArgMatches};
use conllu::io::{ReadSentence, Reader, WriteSentence, Writer};
use stdinout::{Input, Output};
use syntaxdot_encoders::lemma::{BackoffStrategy, EditTree, EditTreeEncoder};
use syntaxdot_encoders::SentenceEncoder;
use udgraph::graph::Node;

use crate::SyntaxDotApp;

static INPUT: &str = "INPUT";
static LABEL_FEATURE: &str = "LABEL_FEATURE";
static OUTPUT: &str = "OUTPUT";

static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];

pub struct Lemma {
    input: Option<String>,
    label_feature: String,
    output: Option<String>,
}

impl SyntaxDotApp for Lemma {
    fn app() -> App<'static, 'static> {
        App::new("lemma")
            .settings(DEFAULT_CLAP_SETTINGS)
            .about("Convert lemmas to edit trees")
            .arg(
                Arg::with_name(LABEL_FEATURE)
                    .short("f")
                    .long("feature")
                    .value_name("NAME")
                    .help("Name of the feature used for the dependency label")
                    .default_value("edit_tree"),
            )
            .arg(Arg::with_name(INPUT).help("Input data").index(1))
            .arg(Arg::with_name(OUTPUT).help("Output data").index(2))
    }

    fn parse(matches: &ArgMatches) -> Result<Self> {
        let input = matches.value_of(INPUT).map(ToOwned::to_owned);
        let label_feature = matches.value_of(LABEL_FEATURE).unwrap().into();
        let output = matches.value_of(OUTPUT).map(ToOwned::to_owned);

        Ok(Lemma {
            input,
            label_feature,
            output,
        })
    }

    fn run(&self) -> Result<()> {
        let input = Input::from(self.input.as_ref());
        let reader = Reader::new(input.buf_read().context("Cannot open input for reading")?);

        let output = Output::from(self.output.as_ref());
        let writer = Writer::new(BufWriter::new(
            output.write().context("Cannot open output for writing")?,
        ));

        self.label_with_encoder(&EditTreeEncoder::new(BackoffStrategy::Form), reader, writer)
    }
}

impl Lemma {
    fn label_with_encoder<R, W>(
        &self,
        encoder: &EditTreeEncoder,
        read: R,
        mut write: W,
    ) -> Result<()>
    where
        R: ReadSentence,
        W: WriteSentence,
    {
        for sentence in read.sentences() {
            let mut sentence = sentence.context("Cannot parse sentence")?;

            let encoded = encoder
                .encode(&sentence)
                .context("Cannot dependency-encode sentence")?;

            for (token, encoding) in sentence.iter_mut().filter_map(Node::token_mut).zip(encoded) {
                token.misc_mut().insert(
                    self.label_feature.clone(),
                    Some(DisplayEditTree(encoding).to_string()),
                );
            }

            write
                .write_sentence(&sentence)
                .context("Cannot write sentence")?;
        }

        Ok(())
    }
}

struct DisplayEditTree(EditTree);

impl ToString for DisplayEditTree {
    fn to_string(&self) -> String {
        edit_tree_to_string(Some(&self.0))
    }
}

fn edit_tree_to_string(edit_tree: Option<&EditTree>) -> String {
    match edit_tree {
        Some(EditTree::MatchNode {
            pre,
            suf,
            left,
            right,
        }) => {
            let left = edit_tree_to_string(left.as_ref().map(AsRef::as_ref));
            let right = edit_tree_to_string(right.as_ref().map(AsRef::as_ref));
            format!("(m {} {} {} {})", pre, suf, left, right)
        }
        Some(EditTree::ReplaceNode {
            replacee,
            replacement,
        }) => {
            format!(
                "(r \"{}\" \"{}\")",
                replacee.iter().collect::<String>(),
                replacement.iter().collect::<String>()
            )
        }
        None => "()".to_string(),
    }
}
