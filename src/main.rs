mod chip8;
use chip8::Chip8;

use std::time::{Duration, Instant};
use std::thread::sleep;


fn main() {
    let mut chip8 = Chip8::new();

    let ibm_rom: Vec<u8> = std::fs::read("D:\\RustProjects\\chip_8_emulator\\test_roms\\2-ibm-logo.ch8")
        .expect("Failed to read ROM file. Check the path!");
    println!("File size: {} bytes", ibm_rom.len());

    chip8.load_rom(&ibm_rom);

    let frame_duration = Duration::from_micros(16666); // 1/60th of a second
    let instructions_per_frame = 10; // 10 * 60 = 600Hz CPU speed

    loop {
        let start_time = Instant::now();

        // 600hz
        for _ in 0..instructions_per_frame {
            chip8.tick();
        }

        // 60hz
        chip8.update_timers();

        chip8.render_console();

        let elapsed = start_time.elapsed();
        if elapsed < frame_duration {
            sleep(frame_duration - elapsed);
        }

    }


}



