extern crate clap_verbosity_flag;
extern crate quicli;
extern crate reqwest;
extern crate structopt;
extern crate unicode_width;
extern crate serde;
extern crate serde_json;

use clap_verbosity_flag::Verbosity;
use quicli::prelude::*;
use structopt::StructOpt;
use unicode_width::UnicodeWidthStr;
use reqwest::blocking;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    verbose: Verbosity,
    #[structopt(long = "server", short = "s", default_value = "localhost:9863")]
    server: String,
    #[structopt(long = "password", short = "p")]
    password: Option<String>,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Query {
        #[structopt(possible_values = &["player", "track", "lyrics", "playlist", "queue"], case_insensitive = true)]
        query_type: String,
    },
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerQuery {
    has_song: bool,
    is_paused: bool,
    volume_percent: u8,
    seekbar_current_position: usize,
    seekbar_current_position_human: String,
    state_percent: f64,
    like_status: String,
    repeat_type: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrackQuery {
    author: String,
    title: String,
    album: String,
    cover: String,
    duration: usize,
    duration_human: String,
    url: String,
    id: String,
    is_video: bool,
    is_advertisement: bool,
    in_library: bool
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LyricsQuery {
    provider: String,
    data: String,
    has_loaded: bool
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistQuery {
    list: Vec<String>
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct QueueQuerySong {
    cover: String,
    title: String,
    author: String,
    duration: String
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueueQuery {
    automix: bool,
    current_index: usize,
    list: Vec<QueueQuerySong>
}

fn print_key_values(pairs: &Vec<(String, String)>) {
    let values: Vec<String> = pairs.into_iter().map(|x| x.0.to_owned()).collect();

    let w = find_name_column_width(&values);

    for pair in pairs {
        let mut name = pair.0.to_owned();

        let mut x: usize = 0;

        let value_arr: Vec<String> = pair.1.split("\n").into_iter().map(|val| {
            x += 1;
            if x == 1 {
                val.to_owned()
            } else {
                let d = " ".repeat(w + 4);
                let r = d + val;
                return r.to_owned();
            }
        }).collect();

        name.push_str(&" ".repeat(w - name.len()));

        println!("{}  : {}", name, value_arr.join("\n"));
    }
}

fn find_name_column_width(names: &Vec<String>) -> usize {
    let mut width = 0;

    for name in names {
        let w = UnicodeWidthStr::width(name.as_str());
        if w > width {
            width = w;
        }
    }

    width
}

fn main() -> CliResult {
    let args: Cli = Cli::from_args();

    let mut url = String::from("http://");
    url.push_str(args.server.as_str());

    match args.command {
        Command::Query { query_type } => {
            let l_type = query_type.to_lowercase();

            url.push_str("/query");

            match l_type.as_str() {
                "player" => {
                    url.push_str("/player");

                    let body: PlayerQuery = blocking::get(&url)?.json()?;

                    print_key_values(&vec![
                        ("Has song".to_owned(), body.has_song.to_string()),
                        ("Is paused".to_owned(), body.is_paused.to_string()),
                        ("Seekbar position".to_owned(), body.seekbar_current_position_human),
                        ("Volume".to_owned(), body.volume_percent.to_string()),
                        ("Percent state".to_owned(), body.state_percent.to_string()),
                        ("Like status".to_owned(), body.like_status),
                        ("Repeat type".to_owned(), body.repeat_type),
                    ]);
                }
                "track" => {
                    url.push_str("/track");

                    let body: TrackQuery = blocking::get(&url)?.json()?;

                    print_key_values(&vec![
                        ("Author".to_owned(), body.author),
                        ("Title".to_owned(), body.title),
                        ("Album".to_owned(), body.album),
                        ("Cover URL".to_owned(), body.cover),
                        ("Duration".to_owned(), body.duration_human),
                        ("URL".to_owned(), body.url),
                        ("ID".to_owned(), body.id),
                        ("Is video".to_owned(), body.is_video.to_string()),
                        ("Is advertisement".to_owned(), body.is_advertisement.to_string()),
                        ("In library".to_owned(), body.in_library.to_string())
                    ]);

                }
                "lyrics" => {
                    url.push_str("/lyrics");

                    let body: LyricsQuery = blocking::get(&url)?.json()?;

                    print_key_values(&vec![
                        ("Loaded".to_owned(), body.has_loaded.to_string()),
                        ("Provider".to_owned(), body.provider),
                        ("Data".to_owned(), body.data)
                    ]);
                }
                "playlist" => {
                    url.push_str("/playlist");

                    let body: PlaylistQuery = blocking::get(&url)?.json()?;

                    print_key_values(&vec![
                        ("List".to_owned(), body.list.join(",\n"))
                    ]);
                }
                "queue" => {
                    url.push_str("/queue");

                    let body: QueueQuery = blocking::get(&url)?.json()?;

                    let mut kvals: Vec<(String, String)> = vec![
                        ("Automix".to_owned(), body.automix.to_string()),
                        ("Current index".to_owned(), body.current_index.to_string())
                    ];

                    let w = body.list.len().to_string().len() + 2;

                    for (index, song) in body.list.into_iter().enumerate() {
                        let mut key1 = (index + 1).to_string();
                        let currw = key1.len();

                        key1.push_str(&" ".repeat(w - currw));
                        key1.push_str("Title");

                        kvals.push((key1.to_owned(), song.title.to_owned()));
                        kvals.push(("    Author".to_owned(), song.author.to_owned()));
                        kvals.push(("    Duration".to_owned(), song.duration.to_owned()));
                        kvals.push(("    Cover".to_owned(), song.cover.to_owned())); 
                    }

                    print_key_values(&kvals);
                }
                _ => {} // should never actually happen so i leave it empty
            };
        }
    };

    Ok(())
}
