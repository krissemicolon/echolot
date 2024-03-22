use std::time::Duration;

use audio_thread_priority::{
    demote_current_thread_from_real_time, promote_current_thread_to_real_time,
};
use rodio::{source::SineWave, Source};

use crate::audio_devices::AudioOutputDevice;

pub fn playback(
    sines: Vec<SineWave>,
    amplification: f32,
    duration: Duration,
    audio_output: &AudioOutputDevice,
) {
    match promote_current_thread_to_real_time(0, 44100) {
        Ok(h) => {
            //println!("this thread is now bumped to real-time priority."); // maybe integrate logging

            sines.into_iter().for_each(|sin| {
                audio_output
                    .sink
                    .append(sin.take_duration(duration).amplify(amplification));
            });
            audio_output.sink.sleep_until_end();

            match demote_current_thread_from_real_time(h) {
                Ok(_) => {
                    //println!("this thread is now bumped back to normal.") // maybe integrate logging
                }
                Err(_) => {
                    panic!("Could not bring the thread back to normal priority.")
                }
            };
        }
        Err(e) => {
            panic!("Error promoting thread to real-time: {}", e);
        }
    }
}
