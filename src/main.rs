use clap::Parser;
use std::io::BufReader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String
}

fn main() {
    let args = Args::parse();

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
