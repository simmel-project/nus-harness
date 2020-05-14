mod controller;
mod fsk;
mod modulator;
mod wav;

extern crate clap;
use clap::{App, Arg};

use std::fs::File;
use std::io::prelude::*;

// const DEFAULT_SAMPLE_RATE: f64 = 48000.0;
const DEFAULT_SAMPLE_RATE: f64 = 44100.0;

#[derive(PartialEq)]
pub enum EncodingRate {
    Low,
    Mid,
    High,
}

impl EncodingRate {
    pub fn silence_divisor(&self) -> u32 {
        match *self {
            EncodingRate::Low => 4,
            EncodingRate::Mid => 2,
            EncodingRate::High => 1,
        }
    }
}

enum ModulationError {
    Io(std::io::Error),
    FloatParse(std::num::ParseFloatError),
    IntParse(std::num::ParseIntError),
}

impl core::fmt::Display for EncodingRate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            EncodingRate::Low => write!(f, "Low"),
            EncodingRate::Mid => write!(f, "Mid"),
            EncodingRate::High => write!(f, "High"),
        }
    }
}

impl std::convert::From<std::io::Error> for ModulationError {
    fn from(error: std::io::Error) -> Self {
        ModulationError::Io(error)
    }
}

impl std::convert::From<std::num::ParseFloatError> for ModulationError {
    fn from(error: std::num::ParseFloatError) -> Self {
        ModulationError::FloatParse(error)
    }
}

impl std::convert::From<std::num::ParseIntError> for ModulationError {
    fn from(error: std::num::ParseIntError) -> Self {
        ModulationError::IntParse(error)
    }
}

impl core::fmt::Debug for ModulationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            ModulationError::Io(e) => write!(f, "Io Error {:?}", e),
            ModulationError::FloatParse(e) => write!(f, "Unable to parse float: {:?}", e),
            ModulationError::IntParse(e) => write!(f, "Unable to parse integer: {:?}", e),
        }
    }
}

struct ModulationConfig {
    data_rate: EncodingRate,
    os_update: bool,
    version: controller::ProtocolVersion,
    silence_prefix: Option<u32>,
    repeat_count: u32,
    sample_rate: f64,
    baud_rate: f64,
    f_lo: f64,
    f_hi: f64,
}

#[repr(C)]
struct ModulationConfigC {
    sample_rate: u32,
    f_lo: u32,
    f_hi: u32,
    filter_width: u32,
    baud_rate: u32,
}

extern "C" {
    fn attempt_demodulation(
        cfg: *const ModulationConfigC,
        samples: *const i16,
        nsamples: u32,
    ) -> u32;
}

fn do_modulation(
    source_filename: &str,
    cfg: &ModulationConfig,
) -> Result<Vec<f64>, std::io::Error> {
    let sample_rate = match cfg.data_rate {
        EncodingRate::Low => cfg.sample_rate * 4.0,
        EncodingRate::Mid => cfg.sample_rate * 2.0,
        EncodingRate::High => cfg.sample_rate * 1.0,
    };
    let mut controller = controller::Controller::new(
        sample_rate,
        cfg.os_update,
        cfg.version,
        cfg.baud_rate,
        cfg.f_lo,
        cfg.f_hi,
    );

    let input_data = {
        let mut input = File::open(source_filename)?;
        let mut input_data: Vec<u8> = vec![];
        input.read_to_end(&mut input_data)?;
        input_data
    };
    let mut audio_data: Vec<f64> = vec![];

    // Add silence, if it's requested.
    if let Some(silence_msec) = cfg.silence_prefix {
        controller.make_silence(silence_msec, &mut audio_data);
    }

    for _ in 0..cfg.repeat_count {
        controller.encode(&input_data, &mut audio_data, &cfg.data_rate);
        let mut pilot_controller = controller::Controller::new(
            cfg.sample_rate,
            cfg.os_update,
            cfg.version,
            cfg.baud_rate,
            cfg.f_lo,
            cfg.f_hi,
        );
        pilot_controller.pilot(&mut audio_data, &cfg.data_rate);
    }
    Ok(audio_data)
}

fn do_play_file(audio_data: Vec<f64>, sample_rate: f64) -> ! {
    let endpoint = cpal::default_endpoint().expect("Failed to get default endpoint");
    let format = endpoint
        .supported_formats()
        .unwrap()
        .next()
        .expect("Failed to get endpoint format")
        .with_max_samples_rate();
    println!("Format selected: {:?}", format);

    let event_loop = cpal::EventLoop::new();
    let voice_id = event_loop.build_voice(&endpoint, &format).unwrap();
    event_loop.play(voice_id);

    let audio_data_len = audio_data.len();
    let mut audio_data_pos = 0;
    let mut overrun_count = 0;

    // Produce a sinusoid of maximum amplitude.
    let mut next_value = || {
        if audio_data_pos >= audio_data_len {
            overrun_count += 1;
            // After 250ms of silence, exit the program.
            if overrun_count > (sample_rate as u32 / 4) {
                use std::process;
                process::exit(0);
            }
            0.0 as f32
        } else {
            let val = audio_data[audio_data_pos];
            audio_data_pos += 1;
            val as f32
        }
    };

    event_loop.run(move |_, buffer| {
        match buffer {
            cpal::UnknownTypeBuffer::U16(mut buffer) => {
                for sample in buffer.chunks_mut(format.channels.len()) {
                    let value = ((next_value() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }

            cpal::UnknownTypeBuffer::I16(mut buffer) => {
                for sample in buffer.chunks_mut(format.channels.len()) {
                    let value = (next_value() * std::i16::MAX as f32) as i16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }

            cpal::UnknownTypeBuffer::F32(mut buffer) => {
                for sample in buffer.chunks_mut(format.channels.len()) {
                    let value = next_value();
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
        };
    });
}
fn main() -> Result<(), ModulationError> {
    let matches = App::new("Love-to-Code Program Modulator")
        .version("1.3")
        .author("Sean Cross <sean@xobs.io>")
        .about("Takes compiled code and modulates it for a Love-to-Code sticker")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILENAME")
                .help("Name of the input file")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILENAME")
                .help("Name of the wave file to write to. The file extension indicates the format: .wav (wave file), .csv (csv log file)"),
        )
        .arg(
            Arg::with_name("sample-rate")
                .short("r")
                .long("rate")
                .value_name("SAMPLE_RATE")
                .help("Sample rate of the output file"),
        )
        .arg(
            Arg::with_name("play")
                .short("w")
                .long("play")
                .help("Play wave audio out the default sound device"),
        )
        .arg(
            Arg::with_name("version")
                .short("p")
                .long("protocol-version")
                .value_name("VERSION")
                .takes_value(true)
                .possible_values(&["1", "2"])
                .default_value("2")
                .help("Data protocol version"),
        )
        .arg(
            Arg::with_name("repeat-count")
                .short("c")
                .long("repeat-count")
                .value_name("COUNT")
                .takes_value(true)
                .default_value("3")
                .help("Number of times to repeat"),
        )
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .takes_value(false)
                .help("Generate an OS update waveform"),
        )
        .arg(
            Arg::with_name("encoding-rate")
                .short("e")
                .long("encoding-rate")
                .possible_values(&["high", "mid", "low"])
                .value_name("RATE")
                .takes_value(true)
                .default_value("high")
                .help("Audio encoding rate"),
        )
        .arg(
            Arg::with_name("baud-rate")
                .short("b")
                .long("baud")
                .value_name("BAUD_RATE")
                .takes_value(true)
                .default_value("8000")
                .help("Baud rate for transmission"),
        )
        .arg(
            Arg::with_name("f-lo")
                .short("l")
                .long("f-lo")
                .aliases(&["flo", "f_lo", "f_low", "f-low", "flow", "f_space"])
                .value_name("F_LO")
                .takes_value(true)
                .default_value("8666")
                .help("Lower frequency used for F_LO / F_SPACE"),
        )
        .arg(
            Arg::with_name("f-hi")
                .short("h")
                .long("f-hi")
                .aliases(&["fhi", "f_hi", "f_high", "f-high", "f_mark"])
                .value_name("F_HI")
                .takes_value(true)
                .default_value("12500")
                .help("Lower frequency used for F_HI / F_MARK"),
        )
        .arg(
            Arg::with_name("filter-width")
                .long("filter-width")
                .value_name("FILTER_WIDTH")
                .takes_value(true)
                .default_value("8")
                .help("Width of the filter to use during demodulation")
        )
        .arg(
            Arg::with_name("silence-prefix")
                .long("silence-prefix")
                .value_name("MSECS")
                .takes_value(true)
                .help("Number of milliseconds of silence to add to the start"),
        )
        .get_matches();

    let source_filename = matches.value_of("input").unwrap();
    let target_filename = matches.value_of("output").unwrap_or("output.wav");
    let os_update = matches.is_present("update");
    let play_file = matches.is_present("play");
    let repeat_count = matches
        .value_of("repeat-count")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let silence_prefix = matches
        .value_of("silence-prefix")
        .map(|s| s.parse::<u32>().unwrap());
    let output_sample_rate = if play_file {
        let endpoint = cpal::default_endpoint().expect("Failed to get default endpoint");
        let format = endpoint
            .supported_formats()
            .unwrap()
            .next()
            .expect("Failed to get endpoint format")
            .with_max_samples_rate();
        format.samples_rate.0 as f64
    } else {
        matches
            .value_of("sample-rate")
            .map(|s| s.parse::<f64>().unwrap())
            .unwrap_or(DEFAULT_SAMPLE_RATE)
    };
    let baud_rate = matches
        .value_of("baud-rate")
        .map(|s| s.parse::<f64>())
        .unwrap()?;
    let f_lo = matches
        .value_of("f-lo")
        .map(|s| s.parse::<f64>())
        .unwrap()?;
    let f_hi = matches
        .value_of("f-hi")
        .map(|s| s.parse::<f64>())
        .unwrap()?;
    let protocol_version = match matches.value_of("version") {
        Some("1") => controller::ProtocolVersion::V1,
        Some("2") => controller::ProtocolVersion::V2,
        Some(x) => panic!("Unrecognized version found: {}", x),
        None => panic!("No protocol version specified"),
    };
    let filter_width = matches.value_of("filter-width").unwrap().parse()?;
    let data_rate = match matches.value_of("encoding-rate") {
        Some("low") => EncodingRate::Low,
        Some("mid") => EncodingRate::Mid,
        Some("high") => EncodingRate::High,
        Some(x) => panic!("Unrecognized rate found: {}", x),
        None => panic!("No valid rate specified"),
    };

    println!("Modulating {} into {}.", source_filename, target_filename);
    println!(
        "Is update? {}  Data rate: {}  Protocol version: {:?}",
        os_update, data_rate, protocol_version
    );

    let mut cfg = ModulationConfig {
        data_rate,
        os_update,
        baud_rate,
        f_lo,
        f_hi,
        silence_prefix,
        version: protocol_version,
        repeat_count: repeat_count,
        sample_rate: output_sample_rate,
    };

    for f_lo in 8000..9000 {
        cfg.f_lo = f_lo as f64;
        let audio_data = do_modulation(source_filename, &cfg)?;

        if play_file {
            do_play_file(audio_data, output_sample_rate);
        }
        let mut output: Vec<i16> = Vec::new();
        for sample in audio_data {
            // Map -1 .. 1 to -32767 .. 32768
            output.push((sample * 32767.0).round() as i16);
        }

        if target_filename == "just-decode" {
            let ccfg = ModulationConfigC {
                sample_rate: cfg.sample_rate as _,
                f_lo: cfg.f_lo as _,
                f_hi: cfg.f_hi as _,
                filter_width: filter_width,
                baud_rate: cfg.baud_rate as _,
            };
            let successes =
                unsafe { attempt_demodulation(&ccfg, output.as_ptr(), output.len() as u32) };
            println!("Attempted demod, got {} successes", successes);
        } else {
            wav::write_wav(cfg.sample_rate as u32, &output, target_filename)?;
        }
    }

    Ok(())
}
