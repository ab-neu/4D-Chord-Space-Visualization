mod engine;
mod midi;
mod rgba;
mod transformation;

use std::env;
//use std::ffi::OsStr;
use std::path::Path;
use std::process;

fn main() {
    // parse args
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("[-.-] Usage: ./visual <path-to-midi-file>");
        process::exit(1);
    }

    // resolve path
    let path = Path::new(&args[1]);
    if !path.exists() {
        eprintln!("[-.-] Path: {:?} does not exist", path);
    } else {
        println!("[^.^] Found midi file at {:?}", path);
    }

    // parse midi file
    let voice_leadings: Vec<[i32; 4]> = midi::parse(path).expect("REASON");

    println!("🎵 Parsed Voice Leadings:");
    for (i, chord) in voice_leadings.iter().enumerate() {
        println!("{:03}: {:?}", i, chord);
    }

    // transform sequence
    let transformation: Vec<[i32; 4]> = transformation::convert(voice_leadings);
    let mut total_shift = [0; 4];
    println!("\n🎹 Transformed Voice Motion Vectors:");
    for (i, vec) in transformation.iter().enumerate() {
        println!("{:03}: {:?}", i, vec);
        for j in 0..4 {
            total_shift[j] += vec[j];
        }
    }
    println!("\n🧮 Total shift [total, x, y, z]: {:?}", total_shift);
    // render sequence
    let start = std::time::Instant::now();
    engine::render(transformation);
    let elapsed = start.elapsed().as_secs_f32();
    println!("Time spent animating: {elapsed}");
}
