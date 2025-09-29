//! # Summary
//!
//! mp3ファイルを再生するアプリ。

use clap::{Parser, Subcommand};
use mp3player::get_playlist;
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
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
    /// プレイリストを再生する
    #[command(arg_required_else_help = true)]
    Play {
        /// プレイリストファイル
        #[arg(long, required = true, short, value_name = "filename")]
        playlist_file: String,
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
        SubCommands::Play { playlist_file } => {
            let result = play(playlist_file);
            println!("result -> {result:?}");
        }
        SubCommands::Stop {} => {
            println!("stop");
        }
    }
}

/// # Summary
///
/// プレイリストに含まれるトラックを再生する
///
/// # Arguments
///
/// - `playlist_file`: プレイリストファイル
///
/// # Returns
///
/// - `Ok(())`: ()
/// - `Err(Box<dyn std::error::Error>)`: エラーメッセージ
fn play(playlist_file: String) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let mut sinks = vec![];

    println!("playlist_file -> {playlist_file:?}");

    let playlist;

    match get_playlist(playlist_file) {
        Ok(json) => {
            println!("json -> {json:?}");
            playlist = json;
        }
        Err(e) => {
            panic!("e -> {e:?}");
        }
    }

    for track in playlist.tracks() {
        let track_file = track.path(playlist.base_path());

        let _ = File::open(&track_file)
            .map_err(|e| {
                let error_message =
                    format!("Failed to open track file: track_file -> {track_file:?}, e -> {e:?}");
                println!("{error_message:?}");
                error_message
            })
            .map(|f| {
                let sink = rodio::Sink::try_new(&stream_handle)?;
                sink.set_volume(playlist.volume() as f32);

                let mut decoder = rodio::Decoder::new(BufReader::new(f))?;
                decoder.try_seek(track.start_position()?)?;

                let mut playback_duration = track.playback_duration()?;

                if playback_duration.is_zero() {
                    // note 0.18.0 から値が取得できるようになっている
                    playback_duration = decoder.total_duration().unwrap_or(Duration::from_secs(0));
                }

                let tmp = decoder.take_duration(playback_duration);

                if playlist.repeat() {
                    sink.append(tmp.repeat_infinite());
                } else {
                    sink.append(tmp);
                }

                sinks.push(sink);

                Ok::<(), Box<dyn std::error::Error>>(())
            });
    }

    //todo 最長のトラックの再生が完了するのを待つように修正
    // 最初のトラックの再生が完了するまで待つ
    sinks[0].sleep_until_end();

    Ok(())
}
