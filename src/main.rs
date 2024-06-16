//! # Summary
//!
//! mp3ファイルを再生するアプリ。

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
        #[arg(default_values_t = ["".to_string()], required = true)]
        files: Vec<String>,
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
        volume: Option<f64>,

        /// Playlist file
        #[arg(exclusive(true), long, value_name = "filename")]
        playlist: Option<String>,
    },
    Stop {},
}

/// # Summary
///
/// エントリポイント
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

/// # Summary
///
/// 指定したファイルを再生する
///
/// # Arguments
///
/// - `files`: 再生対象ファイル
/// - `playlist`: プレイリストファイル
/// - `position`: シーク位置(秒)
/// - `repeat`: リピート再生するかどうか
/// - `skip`: スキップ時間(秒)
/// - `take`: 再生時間(秒)
/// - `volume`: ボリューム(1 を 100% とした数値)
fn play(
    mut files: Vec<String>,
    playlist: Option<String>,
    position: Option<Vec<u64>>,
    repeat: bool,
    skip: Option<Vec<u64>>,
    take: Option<Vec<u64>>,
    volume: Option<f64>,
) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let mut sinks = vec![];

    println!("playlist -> {playlist:?}, position -> {position:?}, repeat -> {repeat:?}, skip -> {skip:?}, take -> {take:?}");
    let playlist = playlist.unwrap_or_default();
    let mut positions = position.unwrap_or_default();
    let skips = skip.unwrap_or_default();
    let mut takes = take.unwrap_or_default();

    let mut i = 0;

    let mut files2: Vec<String> = vec![];
    let mut positions2: Vec<u64> = vec![];
    let mut takes2: Vec<u64> = vec![];
    // parse_json_data()の解析結果を設定するための変数
    let mut repeat2 = false;
    // 最終的に適用される repeat の値
    let mut repeat3 = repeat;
    // parse_json_data()の解析結果を設定するための変数
    let mut volume2 = 1.0;
    // 最終的に適用される volume の値
    let mut volume3 = volume.unwrap_or(1.0);

    match read_json_data(playlist) {
        Ok(json) => {
            println!("json -> {json:?}");
            parse_json_data("", json, &mut files2, &mut positions2, &mut repeat2, &mut takes2, &mut volume2);
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
        repeat3 = repeat2;
        takes = takes2;
        volume3 = volume2;
    }

    for file in &files {
        match File::open(file) {
            Ok(f) => {
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                sink.set_volume(volume3 as f32);

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

                if repeat3 {
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

    //todo 最長のトラックの再生が完了するのを待つように修正
    // 最初のトラックの再生が完了するまで待つ
    sinks[0].sleep_until_end();
}

/// # Summary
///
/// ファイルから JSON データを読み込み、その JSON データを返す
///
/// # Arguments
///
/// - `file`: JSON ファイル
fn read_json_data(file: String) -> Result<Value, serde_json::Error> {
    match File::open(file) {
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

/// # Summary
///
/// プレイリストの JSON データを再帰的にパースして、その結果を各変数に設定する
///
/// # Arguments
///
/// - `key`: JSON データのキー
/// - `value`: JSON データのバリュー
///
/// 以降の引数はパースした結果
///
/// - `files`: 再生対象ファイル
/// - `positions`: シーク位置(秒)
/// - `repeat`: リピート再生するかどうか
/// - `takes`: 再生時間(秒)
/// - `volume`: ボリューム(1 を 100% とした数値)
fn parse_json_data(
    key: &str,
    value: Value,
    files: &mut Vec<String>,
    positions: &mut Vec<u64>,
    repeat: &mut bool,
    takes: &mut Vec<u64>,
    volume: &mut f64,
) {
    println!("key -> {key:?}, value -> {value:?}");

    match value {
        Value::Array(a) => {
            for v in a {
                parse_json_data("0", v, files, positions, repeat, takes, volume);
            }
        }
        Value::Bool(b) => {
            match key {
                "repeat" => {
                    *repeat = b;
                }
                "dummy" => {}
                _ => {}
            }
        }
        Value::Null => {}
        Value::Number(n) => {
            match key {
                "position" => {
                    positions.push(n.as_u64().unwrap_or(0));
                }
                "skip" => {}
                "take" => {
                    takes.push(n.as_u64().unwrap_or(0));
                }
                "volume" => {
                    *volume = n.as_f64().unwrap_or(1.0);
                }
                _ => {}
            }
        }
        Value::Object(o) => {
            for (k, v) in o {
                parse_json_data(k.as_str(), v, files, positions, repeat, takes, volume);
            }
        }
        Value::String(s) => {
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
