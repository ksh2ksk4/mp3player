use clap::Parser;
use std::io::BufReader;

#[derive(Debug, Parser)]
#[command(about = "mp3 player", long_about = None, version = "0.1.0")]
struct Args {
    #[arg(long, short)]
    file: String
}

fn main() {
    let args = Args::parse();
    println!("args -> {args:?}");

    let f = match std::fs::File::open(args.file) {
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
