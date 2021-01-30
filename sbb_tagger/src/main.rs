use sbb_data::*;
use nlpewee::{Sentence, Language};
use nlpewee::error::RequestResult;

use std::error;
use std::collections::HashSet;
use std::sync::Arc;
use clap::{Clap, crate_version, crate_authors, crate_description};
use diesel::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// The path to the services YAML. If omitted, "services.yaml" will be used.
    #[clap(long)]
    services: Option<String>,
    /// The maximum number of robots to tag. If omitted, tag all of the robots in the database.
    #[clap(short, long)]
    limit: Option<usize>,
    /// The maximum number of concurrent requests to the NLPewee server.
    #[clap(short, long, default_value = "255")]
    batch_size: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let opts: Opts = Opts::parse();

    let sc = services::load(opts.services.as_deref())?;
    let sc_nlpewee = sc.nlpewee
        .expect("No NLPewee config found");

    let mut nlpewee_client = nlpewee::ClientBuilder::new();
    nlpewee_client
        .scheme(sc_nlpewee.scheme)
        .socket(sc_nlpewee.host, sc_nlpewee.port);

    let nlpewee_client = nlpewee_client.connect().await?;

    let db_conn = sbb_data::connect_env()?;

    let stop_stems = include_str!("stop_stems");
    let stop_stems = Arc::new(stop_stems
        .split("\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<HashSet<&str>>());

    let mut num_tagged = 0;
    let mut tagged_ids = Vec::<i64>::new();

    loop {
        if let Some(limit) = opts.limit {
            if num_tagged >= limit {
                break;
            }
        }

        let batch_size = if let Some(limit) = opts.limit {
            opts.batch_size.min(limit - num_tagged)
        } else {
            opts.batch_size
        };

        let batch = select_batch(&db_conn, batch_size, &tagged_ids)?;

        if batch.is_empty() {
            break;
        }

        num_tagged += batch.len();

        let mut join_handles = Vec::new();

        for group in batch {
            let mut nlpewee_client = nlpewee_client.clone();
            let stop_stems = stop_stems.clone();
            let group_id = group.id;

            tagged_ids.push(group_id);

            join_handles.push(tokio::spawn(async move {
                tag_group(&mut nlpewee_client, &group, &stop_stems)
                    .await
                    .map(|tags| (group_id, tags))
            }));
        }

        for join_handle in join_handles {
            let (group_id, tags) = join_handle.await??;
            create_tags(&db_conn, group_id, &tags)?;
        }

        println!("{} tagged", num_tagged);
    }

    Ok(())
}

fn select_batch(db_conn: &PgConnection, limit: usize, id_blacklist: &[i64]) -> QueryResult<Vec<RobotGroup>> {
    use schema::*;
    use diesel::dsl::{not, exists};
    robot_groups::table
        .filter(not(exists(tagged_markers::table.filter(tagged_markers::robot_group_id.eq(robot_groups::id)))))
        .filter(not(robot_groups::id.eq_any(id_blacklist)))
        .limit(limit as i64)
        .load(db_conn)
}

fn create_tags(db_conn: &PgConnection, group_id: i64, tag_strings: &[String]) -> QueryResult<()> {
    use schema::*;
    use diesel::dsl::insert_into;

    db_conn.transaction(|| {
        let insert_values = tag_strings
            .iter()
            .map(|tag_string| {
                (tags::robot_group_id.eq(group_id), tags::tag.eq(tag_string.as_str()))
            })
            .collect::<Vec<_>>();

        insert_into(tags::table)
            .values(&insert_values)
            .execute(db_conn)?;

        NewTaggedMarker{
            robot_group_id: group_id,
        }.create(db_conn)?;

        Ok(())
    })
}

async fn tag_group(nlpewee_client: &mut nlpewee::Client, group: &RobotGroup, stop_stems: &HashSet<&str>) -> RequestResult<Vec<String>> {
    let mut sentences = Vec::new();

    let body = nlpewee_client.tokenize(group.body.to_lowercase(), Language::English).await?;
    sentences.extend(body.into_iter());

    if let Some(alt) = group.alt.clone() {
        let alt = nlpewee_client.tokenize(alt, Language::English).await?;
        sentences.extend(alt.into_iter());
    }

    let mut tags = Vec::new();
    let mut tags_set = HashSet::<String>::new();

    for sentence in sentences {
        for sentence_tag in choose_tags(sentence, stop_stems) {
            if !tags_set.contains(&sentence_tag) {
                tags_set.insert(sentence_tag.clone());
                tags.push(sentence_tag);
            }
        }
    }

    Ok(tags)
}

fn choose_tags(sentence: Sentence, stop_stems: &HashSet<&str>) -> Vec<String> {
    lazy_static! {
        static ref BOT_RE: Regex = Regex::new(r".*[Bb][Oo][Tt][Ss]?").unwrap();
    }

    let mut tags = Vec::new();

    let tokens = sentence
        .tokens
        .into_iter()
        .filter(|t| {
            !t.is_stopword() && !t.is_at_mention() && !t.is_hashtag() && !t.is_apostrophe()
                && (t.tag.is_noun() || t.tag.is_verb() || t.tag.is_adjective())
                && !BOT_RE.is_match(&t.full.raw) && !stop_stems.contains(&t.stem.raw.as_str())
        });

    for token in tokens {
        if let Some(full_cleaned) = token.full.cleaned {
            tags.push(full_cleaned);
        }
        if let Some(stem_cleaned) = token.stem.cleaned {
            tags.push(stem_cleaned);
        }
    }

    tags
}
