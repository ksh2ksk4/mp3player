//! # Summary
//!
//! mp3ファイルを再生するアプリ。

use clap::{Parser, Subcommand};
use mp3player::time_string_to_seconds;
use rodio::Source;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
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
        /// Base path of files to play
        #[arg(long, short)]
        base_path: Option<String>,
        /// Seek to position
        #[arg(long, short, value_delimiter = ',', value_name = "time_string")]
        position: Option<Vec<String>>,
        /// Whether to repeat
        #[arg(long, short)]
        repeat: bool,
        /// Skip duration
        #[arg(long, short, value_delimiter = ',', value_name = "s")]
        skip: Option<Vec<u64>>,
        /// Take duration
        #[arg(long, short, value_delimiter = ',', value_name = "time_string")]
        take: Option<Vec<String>>,
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
            base_path,
            position,
            repeat,
            skip,
            take,
            volume,
        } => {
            println!("files -> {files:?}, volume -> {volume:?}");
            let result = play(
                files, playlist, base_path, position, repeat, skip, take, volume,
            );
            println!("result -> {result:?}");
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
/// - `base_path`: 再生対象ファイルのベースパス
/// - `position`: シーク位置(時刻文字列)
/// - `repeat`: リピート再生するかどうか
/// - `skip`: スキップ時間(秒)
/// - `take`: 再生時間(時刻文字列)
/// - `volume`: ボリューム(1 を 100% とした数値)
///
/// # Returns
///
/// - `Ok(())`: ()
/// - `Err(Box<dyn std::error::Error>)`: エラーメッセージ
fn play(
    mut files: Vec<String>,
    playlist: Option<String>,
    base_path: Option<String>,
    position: Option<Vec<String>>,
    repeat: bool,
    skip: Option<Vec<u64>>,
    take: Option<Vec<String>>,
    volume: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let mut sinks = vec![];

    println!("playlist -> {playlist:?}, base_path -> {base_path:?}, position -> {position:?}, repeat -> {repeat:?}, skip -> {skip:?}, take -> {take:?}");
    let playlist = playlist.unwrap_or_default();
    let base_path = base_path.unwrap_or_default();
    let mut positions = position.unwrap_or_default();
    let skips = skip.unwrap_or_default();
    let mut takes = take.unwrap_or_default();

    let mut i = 0;

    // parse_json_data()の解析結果を設定するための変数
    let mut base_path2 = "".to_string();
    // 最終的に適用される repeat の値
    let mut base_path3 = base_path;

    let mut files2: Vec<String> = vec![];
    let mut positions2: Vec<String> = vec![];
    let mut takes2: Vec<String> = vec![];

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
            parse_json_data(
                "",
                json,
                &mut base_path2,
                &mut files2,
                &mut positions2,
                &mut repeat2,
                &mut takes2,
                &mut volume2,
            );
            println!("files2 -> {files2:?}, positions2 -> {positions2:?}");
        }
        Err(e) => {
            println!("e -> {e:?}");
            panic!("{}", e.to_string());
        }
    }

    if !files2.is_empty() {
        base_path3 = base_path2;
        files = files2;
        positions = positions2;
        repeat3 = repeat2;
        takes = takes2;
        volume3 = volume2;
    }

    for file in &files {
        let path = Path::new(&base_path3).join(file);

        match File::open(&path) {
            Ok(f) => {
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                sink.set_volume(volume3 as f32);

                let mut decoder = rodio::Decoder::new(BufReader::new(f)).unwrap();
                // 0.18.0 から値が取得できるようになっている
                let total_duration = decoder.total_duration().unwrap_or(Duration::from_secs(0));
                println!("total_duration -> {total_duration:?}");

                decoder.try_seek(Duration::from_secs(time_string_to_seconds(&positions[i])?))?;

                let tmp = decoder
                    .skip_duration(Duration::from_secs(if skips.is_empty() {
                        0
                    } else {
                        skips[i]
                    }))
                    .take_duration(if takes.is_empty() {
                        total_duration
                    } else {
                        Duration::from_secs(time_string_to_seconds(&takes[i])?)
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

    Ok(())
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
/// - `base_path`: 再生対象ファイルのベースパス
/// - `files`: 再生対象ファイル
/// - `positions`: シーク位置(時刻文字列)
/// - `repeat`: リピート再生するかどうか
/// - `takes`: 再生時間(時刻文字列)
/// - `volume`: ボリューム(1 を 100% とした数値)
fn parse_json_data(
    key: &str,
    value: Value,
    base_path: &mut String,
    files: &mut Vec<String>,
    positions: &mut Vec<String>,
    repeat: &mut bool,
    takes: &mut Vec<String>,
    volume: &mut f64,
) {
    println!("key -> {key:?}, value -> {value:?}");

    match value {
        Value::Array(a) => {
            for v in a {
                parse_json_data("0", v, base_path, files, positions, repeat, takes, volume);
            }
        }
        Value::Bool(b) => match key {
            "repeat" => {
                *repeat = b;
            }
            "dummy" => {}
            _ => {}
        },
        Value::Null => {}
        Value::Number(n) => match key {
            "skip" => {}
            "volume" => {
                *volume = n.as_f64().unwrap_or(1.0);
            }
            _ => {}
        },
        Value::Object(o) => {
            for (k, v) in o {
                parse_json_data(
                    k.as_str(),
                    v,
                    base_path,
                    files,
                    positions,
                    repeat,
                    takes,
                    volume,
                );
            }
        }
        Value::String(s) => match key {
            "base_path" => {
                *base_path = s;
            }
            "file" => {
                files.push(s);
            }
            "dummy" => {}
            "position" => {
                positions.push(s);
            }
            "take" => {
                takes.push(s);
            }
            _ => {}
        },
    }
}
