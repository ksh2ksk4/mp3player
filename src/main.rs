use clap::{Parser, Subcommand};
use rodio::Source;
use std::fs::File;
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
    /// Play the specified file
    #[command(arg_required_else_help = true)]
    Play {
        /// Files to play
        #[arg(required = true)]
        files: Vec<String>
    },
    Stop {

    }
}

fn main() {
    let args = Cli::parse();
    println!("args -> {args:?}");

    match args.command {
        SubCommands::Play {
            files
        } => {
            println!("files -> {files:?}");
            play(files);
        },
        SubCommands::Stop {} => {
            println!("stop");
        }
    }
}

fn play(files: Vec<String>) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    for file in &files {
        match File::open(file) {
            Ok(f) => {
                let _ = stream_handle.play_raw(
                    rodio::Decoder::new(
                        BufReader::new(f)
                    ).unwrap().convert_samples()
                );
            },
            Err(error) => {
                println!("error -> {error:?}");
                panic!("{}", error.to_string());
            }
        };
    }

    std::thread::sleep(std::time::Duration::from_secs(10));
}
