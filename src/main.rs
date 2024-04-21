use std::io::BufReader;

fn main() {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    sink.append(rodio::Decoder::new(BufReader::new(
        std::fs::File::open("assets/music.mp3").unwrap()
    )).unwrap());
    sink.sleep_until_end();
}
