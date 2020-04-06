use rodio::buffer::SamplesBuffer;
use std::io::{BufRead, BufReader};

fn main() {
    let mut samples: Vec<i16> = Vec::new();

    for line in BufReader::new(std::io::stdin()).lines() {
        let line = line.unwrap();
        samples.push(line.trim().parse().unwrap())
    }

    println!("Playing {} samples", samples.len());

    let device = rodio::default_output_device().unwrap();
    let sink = rodio::Sink::new(&device);

    sink.append(SamplesBuffer::new(1, 44100, samples));

    sink.sleep_until_end();
}
