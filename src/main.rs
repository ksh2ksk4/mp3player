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
    command: SubCommands
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
        #[arg(long, short, value_name = "s")]
        skip: Option<u64>,
        /// Take duration
        #[arg(long, short, value_name = "s")]
        take: Option<u64>,
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

    match args.command {
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

fn play(files: Vec<String>, skip: Option<u64>, take: Option<u64>, volume: Option<f32>) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    let mut sinks = Vec::new();

    for file in &files {
        match File::open(file) {
            Ok(f) => {
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                sink.set_volume(volume.unwrap_or(1.0));

                let decoder = rodio::Decoder::new(BufReader::new(f)).unwrap();
                let total_duration = decoder.total_duration();
                println!("total_duration -> {total_duration:?}");
                sink.append(
                    decoder.skip_duration(Duration::from_secs(skip.unwrap_or(0)))
                        //todo デフォルトをtotal_durationの値にしたい
                        .take_duration(Duration::from_secs(take.unwrap_or(30)))
                );

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

    sinks[0].sleep_until_end();
}
