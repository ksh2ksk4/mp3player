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
            position,
            repeat,
            skip,
            take,
            volume,
        } => {
            println!("files -> {files:?}, volume -> {volume:?}");
            play(files, position, repeat, skip, take, volume);
        }
        SubCommands::Stop {} => {
            println!("stop");
        }
    }
}

fn play(
    files: Vec<String>,
    position: Option<Vec<u64>>,
    repeat: bool,
    skip: Option<Vec<u64>>,
    take: Option<Vec<u64>>,
    volume: Option<f32>,
) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let mut sinks = vec![];

    println!("position -> {position:?}, repeat -> {repeat:?}, skip -> {skip:?}, take -> {take:?}");
    let positions = position.unwrap_or_default();
    let skips = skip.unwrap_or_default();
    let takes = take.unwrap_or_default();

    let mut i = 0;

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

                //todo これをもっと簡潔に
                if repeat {
                    sink.append(
                        decoder
                            .skip_duration(Duration::from_secs(if skips.is_empty() {
                                0
                            } else {
                                skips[i]
                            }))
                            .take_duration(if takes.is_empty() {
                                total_duration
                            } else {
                                Duration::from_secs(takes[i])
                            })
                            .repeat_infinite(),
                    );
                } else {
                    sink.append(
                        decoder
                            .skip_duration(Duration::from_secs(if skips.is_empty() {
                                0
                            } else {
                                skips[i]
                            }))
                            .take_duration(if takes.is_empty() {
                                total_duration
                            } else {
                                Duration::from_secs(takes[i])
                            }),
                    );
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
        };
    }

    // 最初のトラックの再生が完了するまで待つ
    sinks[0].sleep_until_end();
}
