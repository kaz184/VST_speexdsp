#[macro_use]
extern crate vst;
extern crate dlopen;
#[macro_use]
extern crate dlopen_derive;
extern crate libc;
#[macro_use]
extern crate itertools;

mod speexdsp;

use libc::{ c_int, c_short };
use speexdsp::{ SpeexPreProcess };
use vst::plugin::{ Info, Plugin, Category };
use vst::buffer::AudioBuffer;
use std::collections::VecDeque;

const SAMPLE_RATE: c_int = 48_000;
const FRAME_SIZE: usize = (SAMPLE_RATE as usize) / 50;

fn normalize(x: f32, from0: f32, to0:f32, from1: f32, to1: f32) -> f32 {
    let x = (x - from0) / (to0 - from0);
    x * (to1 - from1) + from1
}

fn dB_to_unity(x: i32) -> f32 {
    normalize(x as f32, -90.0, 30.0, 0.0, 1.0)
}

fn unity_to_dB(x: f32) -> i32 {
    normalize(x, 0.0, 1.0, -90.0, 30.0) as i32
}

struct Mumble {
    input_buffer: VecDeque<f32>,
    output_buffer: VecDeque<f32>,
    processor: SpeexPreProcess,

    noise_suppression: f32
}

impl Default for Mumble {
    fn default() -> Self {
        Mumble::new()
    }
}

impl Mumble {
    fn new() -> Mumble {
        Mumble {
            input_buffer: VecDeque::with_capacity(100),
            output_buffer: VecDeque::with_capacity(100),
            processor: Mumble::make_processor(),

            noise_suppression: 0.0
        }
    }

    fn make_processor() -> SpeexPreProcess {
        let mut processor = SpeexPreProcess::new(FRAME_SIZE as i32, SAMPLE_RATE);
        processor.set_AGC_TARGET(30_000);
        processor.set_AGC_LEVEL(30_000.0);
        processor.set_AGC_MAX_GAIN(12);
        processor
    }

    fn add_sample(&mut self, sample: f32) {
        let p = &mut self.processor;

        self.input_buffer.push_back(sample);

        if self.input_buffer.len() >= FRAME_SIZE {
            let mut buffer = self.input_buffer
                .drain(0..FRAME_SIZE)
                .map(|x| (x * 32767.0) as c_short)
                .collect::<Vec<c_short>>();

            let agc_gain = p.get_AGC_GAIN();
            p.set_NOISE_SUPPRESS(unity_to_dB(self.noise_suppression) - agc_gain);

            p.run(buffer.as_mut_ptr());

            for x in buffer {
                self.output_buffer.push_back((x as f32) / 32767.0)
            }
        }
    }

    fn get_sample(&mut self) -> f32 {
        self.output_buffer.pop_front().unwrap_or(0.0)
    }
}

impl Plugin for Mumble {
    fn get_info(&self) -> Info {
        Info {
            name: "Mumble".to_string(),
            unique_id: 11111,
            inputs: 2,
            outputs: 2,
            category: Category::Synth,
            parameters: 14,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (inputs, mut outputs) = buffer.split();

        let (l, r) = inputs.split_at(1);
        let stereo_in = l[0].iter().zip(r[0].iter());

        let (mut l, mut r) = outputs.split_at_mut(1);
        let stereo_out = l[0].iter_mut().zip(r[0].iter_mut());

        for ((left_in, right_in), (left_out, right_out)) in izip!(stereo_in, stereo_out) {
            let sample = (*left_in + *right_in) * 0.5;
            self.add_sample(sample);
            let x = self.get_sample();
            *left_out = x;
            *right_out = x;
        }

    }

    fn get_parameter(&self, index: i32) -> f32 {
        let p = & self.processor ;

        let normalize = |x, f, t| normalize(x as f32, f, t, 0.0, 1.0);
        let bool_ = |x| if x {1.0} else {0.0};
        let prob = |x| normalize(x as f32, 0.0, 100.0);
        let dB = dB_to_unity;
        let level = |x| normalize(x as f32, 0.0, 32767.0);

        match index {
            0  => bool_(       p.get_AGC()),
            1  => dB(          p.get_AGC_DECREMENT()),
            2  => dB(          p.get_AGC_INCREMENT()),
            3  => level(     p.get_AGC_LEVEL()),
            4  => dB(          p.get_AGC_MAX_GAIN()),
            5  => level(     (p.get_AGC_TARGET()) as f32),
            6  => bool_(       p.get_DENOISE()),
            7  => bool_(       p.get_DEREVERB()),
            8  =>              p.get_DEREVERB_DECAY(),
            9  =>              p.get_DEREVERB_LEVEL(),
            10 => self.noise_suppression, // dB(          p.get_NOISE_SUPPRESS()),
            11 => prob(        p.get_PROB_CONTINUE()),
            12 => prob(        p.get_PROB_START()),
            13 => bool_(       p.get_VAD()),

            // 14 => level(     (p.get_AGC_LOUDNESS()) as f32),
            // 15 => prob(        p.get_PROB()),
            // 16 => dB(          p.get_AGC_GAIN()),

            _ => 0.0
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        let p = &mut self.processor;

        let normalize = |x, f, t| normalize(x, 0.0, 1.0, f, t);
        let bool_ = |x| if x > 0.5 {true} else {false};
        let dB = unity_to_dB;
        let level = |x| normalize(x, 0.0, 32767.0);
        let prob = |x| normalize(x, 0.0, 100.0) as i32;
        let ignore = |_| ();
            
        match index {
            0  => ignore (p.set_AGC            (bool_   (value))),
            1  => ignore (p.set_AGC_DECREMENT  (dB      (value))),
            2  => ignore (p.set_AGC_INCREMENT  (dB      (value))),
            3  => ignore (p.set_AGC_LEVEL      (level (value))),
            4  => ignore (p.set_AGC_MAX_GAIN   (dB      (value))),
            5  => ignore (p.set_AGC_TARGET     (level (value) as i32)),
            6  => ignore (p.set_DENOISE        (bool_   (value))),
            7  => ignore (p.set_DEREVERB       (bool_   (value))),
            8  => ignore (p.set_DEREVERB_DECAY (value)),
            9  => ignore (p.set_DEREVERB_LEVEL (value)),
            10 => {
                self.noise_suppression = value;
            }, //ignore (p.set_NOISE_SUPPRESS (dB      (value))),
            11 => ignore (p.set_PROB_CONTINUE  (prob    (value))),
            12 => ignore (p.set_PROB_START     (prob    (value))),
            13 => ignore (p.set_VAD            (bool_   (value))),
            _  => ()
        };
    }

    fn get_parameter_text(&self, index: i32) -> String {
        let normalize = |x, f, t| normalize(x, 0.0, 1.0, f, t);
        let bool_ = |x| if x > 0.5 {true} else {false};
        let dB = unity_to_dB;
        let c_short = |x| normalize(x, 0.0, 32767.0) as i32;
        let prob = |x| normalize(x, 0.0, 100.0) as i32;

        let value = self.get_parameter(index);

        match index {
            0  => format! ("{}", bool_   (value)),
            1  => format! ("{}dB", dB      (value)),
            2  => format! ("{}dB", dB      (value)),
            3  => format! ("{}", c_short (value)),
            4  => format! ("{}dB", dB      (value)),
            5  => format! ("{}", c_short (value)),
            6  => format! ("{}", bool_   (value)),
            7  => format! ("{}", bool_   (value)),
            8  => format! ("{}", value),
            9  => format! ("{}", value),
            10 => format! ("{}dB", dB      (value)),
            11 => format! ("{}%", prob    (value)),
            12 => format! ("{}%", prob    (value)),
            13 => format! ("{}", bool_   (value)),

            // 14 => format! ("{}", c_short( value)),
            // 15 => format! ("{}", prob( value)),
            // 16 => format! ("{}", dB(value)),

            _ => "".to_string()
        }
    }

    fn can_be_automated(&self, index: i32) -> bool {
        match index {
            0...13 => true,
            _ => false
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "AGC",
            1 => "AGC_DECREMENT",
            2 => "AGC_INCREMENT",
            3 => "AGC_LEVEL",
            4 => "AGC_MAX_GAIN",
            5 => "AGC_TARGET",
            6 => "DENOISE",
            7 => "DEREVERB",
            8 => "DEREVERB_DECAY",
            9 => "DEREVERB_LEVEL",
            10 => "NOISE_SUPPRESS",
            11 => "PROB_CONTINUE",
            12 => "PROB_START",
            13 => "VAD",
            // 14 => "AGC_LOUDNESS",
            // 15 => "PROB",
            // 16 => "AGC_GAIN",

            _ => ""
        }.to_string()
    }
}

plugin_main!(Mumble);
