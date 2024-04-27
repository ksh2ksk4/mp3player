use clap::{Parser, Subcommand};
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(name = "mp3player")]
#[command(about = "mp3 player", long_about = None, version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    sub_command: SubCommands
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Play the specified file
    #[command(arg_required_else_help = true)]
    Play {
        /// Files to play
        #[arg(required = true)]
        files: Vec<String>,
        /// Skip duration
        #[arg(long, short, value_delimiter = ',', value_name = "s")]
        skip: Option<Vec<u64>>,
        /// Take duration
        #[arg(long, short, value_delimiter = ',', value_name = "s")]
        take: Option<Vec<u64>>,
        /// Volume of the sound
        #[arg(long, short, value_name = "n")]
        volume: Option<f32>
    },
    Stop {

    }
}

fn main() {
    let args = Cli::parse();
    println!("args -> {args:?}");

    match args.sub_command {
        SubCommands::Play {
            files,
            skip,
            take,
            volume
        } => {
            println!("files -> {files:?}, volume -> {volume:?}");
            play(files, skip, take, volume);
        },
        SubCommands::Stop {} => {
            println!("stop");
        }
    }
}

fn play(files: Vec<String>, skip: Option<Vec<u64>>, take: Option<Vec<u64>>, volume: Option<f32>) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let mut sinks = Vec::new();

    println!("skip -> {skip:?}, take -> {take:?}");
    let skips = skip.unwrap_or([].to_vec());
    //todo デフォルトをtotal_durationの値にしたい
    let takes = take.unwrap_or([].to_vec());

    let mut i = 0;

    for file in &files {
        match File::open(file) {
            Ok(f) => {
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                sink.set_volume(volume.unwrap_or(1.0));

                let decoder = rodio::Decoder::new(BufReader::new(f)).unwrap();
                let total_duration = decoder.total_duration();
                println!("total_duration -> {total_duration:?}");

                sink.append(
                    decoder.skip_duration(Duration::from_secs(skips[i]))
                        .take_duration(Duration::from_secs(takes[i]))
                );
                i += 1;

                let length = sink.len();
                println!("len() -> {length:?}");

                sinks.push(sink);
            },
            Err(error) => {
                println!("error -> {error:?}");
                panic!("{}", error.to_string());
            }
        };
    }

    // 最初のトラックの再生が完了するまで待つ
    sinks[0].sleep_until_end();
}
