use clap::{Parser, Subcommand};
use rodio::Source;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(name = "mp3player")]
#[command(about = "mp3 player", long_about = None, version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    sub_command: SubCommands,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Play the specified file
    #[command(arg_required_else_help = true)]
    Play {
        /// Files to play
        #[arg(required = true)]
        files: Vec<String>,
        /// Playlist file
        #[arg(long, value_name = "filename")]
        playlist: Option<String>,
        /// Seek to position
        #[arg(long, short, value_delimiter = ',', value_name = "s")]
        position: Option<Vec<u64>>,
        /// Whether to repeat
        #[arg(long, short)]
        repeat: bool,
        /// Skip duration
        #[arg(long, short, value_delimiter = ',', value_name = "s")]
        skip: Option<Vec<u64>>,
        /// Take duration
        #[arg(long, short, value_delimiter = ',', value_name = "s")]
        take: Option<Vec<u64>>,
        /// Volume of the sound
        #[arg(long, short, value_name = "n")]
        volume: Option<f32>,
    },
    Stop {},
}

fn main() {
    let args = Cli::parse();
    println!("args -> {args:?}");

    match args.sub_command {
        SubCommands::Play {
            files,
            playlist,
            position,
            repeat,
            skip,
            take,
            volume,
        } => {
            println!("files -> {files:?}, volume -> {volume:?}");
            play(files, playlist, position, repeat, skip, take, volume);
        }
        SubCommands::Stop {} => {
            println!("stop");
        }
    }
}

fn play(
    mut files: Vec<String>,
    playlist: Option<String>,
    position: Option<Vec<u64>>,
    repeat: bool,
    skip: Option<Vec<u64>>,
    take: Option<Vec<u64>>,
    volume: Option<f32>,
) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let mut sinks = vec![];

    println!("playlist -> {playlist:?}, position -> {position:?}, repeat -> {repeat:?}, skip -> {skip:?}, take -> {take:?}");
    let playlist = playlist.unwrap_or_default();
    let mut positions = position.unwrap_or_default();
    let skips = skip.unwrap_or_default();
    let takes = take.unwrap_or_default();

    let mut i = 0;

    let mut files2: Vec<String> = vec![];
    let mut positions2: Vec<u64> = vec![];

    match read_json_data(playlist) {
        Ok(json) => {
            println!("json -> {json:?}");
            parse_json_data("", json, &mut files2, &mut positions2);
            println!("files2 -> {files2:?}, positions2 -> {positions2:?}");
        }
        Err(e) => {
            println!("e -> {e:?}");
            panic!("{}", e.to_string());
        }
    }

    if !files2.is_empty() {
        files = files2;
        positions = positions2;
    }

    for file in &files {
        match File::open(file) {
            Ok(f) => {
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                sink.set_volume(volume.unwrap_or(1.0));

                let mut decoder = rodio::Decoder::new(BufReader::new(f)).unwrap();
                // 0.18.0 から値が取得できるようになっている
                let total_duration = decoder.total_duration().unwrap_or(Duration::from_secs(0));
                println!("total_duration -> {total_duration:?}");

                decoder.try_seek(Duration::from_secs(positions[i])).unwrap();

                let tmp = decoder
                    .skip_duration(Duration::from_secs(if skips.is_empty() {
                        0
                    } else {
                        skips[i]
                    }))
                    .take_duration(if takes.is_empty() {
                        total_duration
                    } else {
                        Duration::from_secs(takes[i])
                    });

                if repeat {
                    sink.append(tmp.repeat_infinite());
                } else {
                    sink.append(tmp);
                }

                i += 1;

                let length = sink.len();
                println!("len() -> {length:?}");

                sinks.push(sink);
            }
            Err(error) => {
                println!("error -> {error:?}");
                panic!("{}", error.to_string());
            }
        }
    }

    // 最初のトラックの再生が完了するまで待つ
    sinks[0].sleep_until_end();
}

fn read_json_data(playlist: String) -> Result<Value, serde_json::Error> {
    match File::open(playlist) {
        Ok(mut f) => {
            let mut contents = String::new();

            f.read_to_string(&mut contents).unwrap();
            serde_json::from_str::<Value>(&contents)
        }
        Err(e) => {
            println!("e -> {e:?}");
            panic!("{}", e.to_string());
        }
    }
}

fn parse_json_data(key: &str, value: Value, files: &mut Vec<String>, positions: &mut Vec<u64>) {
    match value {
        Value::Array(a) => {
            println!("key -> {key:?}, a -> {a:?}");

            for v in a {
                parse_json_data("0", v, files, positions);
            }
        }
        Value::Bool(b) => {
            println!("key -> {key:?}, b -> {b:?}");
        }
        Value::Null => {}
        Value::Number(n) => {
            println!("key -> {key:?}, n -> {n:?}");

            match key {
                "position" => {
                    positions.push(n.as_u64().unwrap_or(0));
                }
                "skip" => {

                }
                "take" => {

                }
                _ => {}
            }
        }
        Value::Object(o) => {
            println!("key -> {key:?}, o -> {o:?}");

            for (k, v) in o {
                parse_json_data(k.as_str(), v, files, positions);
            }
        }
        Value::String(s) => {
            println!("key -> {key:?}, s -> {s:?}");

            match key {
                "file" => {
                    files.push(s);
                }
                "dummy" => {}
                _ => {}
            }
        }
    }
}
