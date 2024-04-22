use clap::{Parser, Subcommand};
use std::io::BufReader;

#[derive(Debug, Parser)]
#[command(name = "mp3player")]
#[command(about = "mp3 player", long_about = None, version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: SubCommands
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    #[command(arg_required_else_help = true)]
    Play {
        file: String
    },
    Stop {

    }
}

fn main() {
    let args = Cli::parse();
    println!("args -> {args:?}");

    match args.command {
        SubCommands::Play { file } => {
            println!("play {file}");
            play(file);
        },
        SubCommands::Stop {} => {
            println!("stop");
        }
    }
}

fn play(file: String) {
    let f = match std::fs::File::open(file) {
        Ok(f) => f,
        Err(error) => {
            println!("error -> {error:?}");
            panic!("{}", error.to_string());
        }
    };

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    sink.append(rodio::Decoder::new(BufReader::new(f)).unwrap());
    sink.sleep_until_end();
}
