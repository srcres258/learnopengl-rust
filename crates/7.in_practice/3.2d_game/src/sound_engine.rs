use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub struct SoundEngine {
    #[allow(dead_code)]
    stream: OutputStream,
    handle: OutputStreamHandle
}

impl SoundEngine {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        Self {
            stream,
            handle
        }
    }

    pub fn play(&self, path: &str, repeat: bool) {
        let file = File::open(path).unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();
        let sink = Sink::try_new(&self.handle).unwrap();
        if repeat {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }
    }
}