use libc::{ c_short, c_int, c_void, c_float };
use dlopen::wrapper::{ Container, WrapperApi };
use std::{ ptr, panic };

#[derive(Debug)]
#[repr(C)]
pub struct SpeexPreProcessState {
    // frame_size: c_int,
    // ps_size: c_int,
    // sampling_rate: c_int,
    // nbands: c_int,
    // bank: *mut c_void,

    // denoise_enabled: c_int,
    // vad_enabled: c_int,
    // dereverb_enabled: c_int,
    // reverb_decay: c_short,
    // reverb_level: c_short,
}

#[repr(C)]
pub struct SpeexEchoState {}

#[derive(WrapperApi)]
pub struct Api {
    speex_preprocess_ctl
        : unsafe extern "C" fn(st: *mut SpeexPreProcessState, request: c_int, ptr: *mut c_void) -> c_int,
    speex_preprocess_estimate_update
        : unsafe extern "C" fn(st: *mut SpeexPreProcessState, x: *mut c_short),
    speex_preprocess_run
        : unsafe extern "C" fn(st: *mut SpeexPreProcessState, x: *mut c_short) -> c_int,
    speex_preprocess_state_destroy
        : unsafe extern "C" fn(st: *mut SpeexPreProcessState),
    speex_preprocess_state_init
        : unsafe extern "C" fn(frame_size: c_int, sampling_rate: c_int) -> *mut SpeexPreProcessState,
}

#[allow(unused)]
pub mod SPEEX {
    pub mod PREPROCESS {
        pub enum GET {
            AGC                  = 3,
            AGC_DECREMENT        = 29,
            AGC_GAIN             = 35,
            AGC_INCREMENT        = 27,
            AGC_LEVEL            = 7,
            AGC_LOUDNESS         = 33,
            AGC_MAX_GAIN         = 31,
            AGC_TARGET           = 47,
            DENOISE              = 1,
            DEREVERB             = 9,
            DEREVERB_DECAY       = 13,
            DEREVERB_LEVEL       = 11,
            ECHO_STATE           = 25,
            ECHO_SUPPRESS        = 21,
            ECHO_SUPPRESS_ACTIVE = 23,
            NOISE_PSD            = 43,
            NOISE_PSD_SIZE       = 41,
            NOISE_SUPPRESS       = 19,
            PROB                 = 45,
            PROB_CONTINUE        = 17,
            PROB_START           = 15,
            PSD                  = 39,
            PSD_SIZE             = 37,
            VAD                  = 5,
        }

        pub enum SET {
            AGC                  = 2,
            AGC_DECREMENT        = 28,
            AGC_INCREMENT        = 26,
            AGC_LEVEL            = 6,
            AGC_MAX_GAIN         = 30,
            AGC_TARGET           = 46,
            DENOISE              = 0,
            DEREVERB             = 8,
            DEREVERB_DECAY       = 12,
            DEREVERB_LEVEL       = 10,
            ECHO_STATE           = 24,
            ECHO_SUPPRESS        = 20,
            ECHO_SUPPRESS_ACTIVE = 22,
            NOISE_SUPPRESS       = 18,
            PROB_CONTINUE        = 16,
            PROB_START           = 14,
            VAD                  = 4,
        }
    }
}

use speexdsp::SPEEX::PREPROCESS::{ GET, SET };

// #[derive(Default)]
pub struct SpeexPreProcess {
    pub lib: Container<Api>,
    pub state: *mut SpeexPreProcessState
}

// const LIB_PATH: &'static str = "D:\\Program Files\\VSTPlugins\\libspeexdsp-1.dll";
const LIB_PATH: &'static str = "libspeexdsp-1.dll";

impl SpeexPreProcess {
    pub fn new(frame_size: i32, sample_rate: i32) -> SpeexPreProcess {
        let lib: Container<Api> =
            unsafe { Container::load(LIB_PATH) }
            .expect(&format!("Could not open library or load symbols: {}", LIB_PATH));

        let state = unsafe { lib.speex_preprocess_state_init(frame_size, sample_rate) };

        SpeexPreProcess {
            lib: lib,
            state: state
        }
    }

    pub fn run(&mut self, x: *mut c_short) -> i32 {
        unsafe { self.lib.speex_preprocess_run(self.state, x) }
    }

    unsafe fn set_(&mut self, request: c_int, arg: *mut c_void) -> c_int {
        self.lib.speex_preprocess_ctl(self.state, request, arg)
    }

    unsafe fn get_(& self, request: c_int, arg: *mut c_void) -> c_int {
        self.lib.speex_preprocess_ctl(self.state, request, arg)
    } 

    unsafe fn set_i32(&mut self, request: SET, arg: c_int) {
        let arg: *mut c_int = &mut arg.clone();
        self.set_(request as c_int , arg as *mut c_void);
    }

    unsafe fn set_f32(&mut self, request: SET, arg: c_float) {
        let arg: *mut c_float = &mut arg.clone();
        self.set_(request as c_int, arg as *mut c_void);
    }

    unsafe fn set_bool(&mut self, request: SET, arg: bool) {
        self.set_i32(request, arg as c_int)
    }

    unsafe fn get_i32(& self, request: GET) -> c_int {
        let arg: *mut c_int = &mut 0;
        self.get_(request as c_int, arg as *mut c_void);
        *arg
    }

    unsafe fn get_f32(& self, request: GET) -> c_float {
        let arg: *mut c_float = &mut 0.0;
        self.get_(request as c_int, arg as *mut c_void);
        *arg
    }

    unsafe fn get_bool(& self, request: GET) -> bool {
        if self.get_i32(request) == 1 { true } else { false }
    }

    pub fn get_AGC(& self)                  -> bool { unsafe { self.get_bool(GET::AGC) } }
    pub fn get_AGC_DECREMENT(& self)        -> i32 { unsafe { self.get_i32(GET::AGC_DECREMENT) } }
    pub fn get_AGC_GAIN(& self)             -> i32 { unsafe { self.get_i32(GET::AGC_GAIN) } }
    pub fn get_AGC_INCREMENT(& self)        -> i32 { unsafe { self.get_i32(GET::AGC_INCREMENT) } }
    pub fn get_AGC_LEVEL(& self)            -> f32 { unsafe { self.get_f32(GET::AGC_LEVEL) } }
    pub fn get_AGC_LOUDNESS(& self)         -> i32 { unsafe { self.get_i32(GET::AGC_LOUDNESS) } }
    pub fn get_AGC_MAX_GAIN(& self)         -> i32 { unsafe { self.get_i32(GET::AGC_MAX_GAIN) } }
    pub fn get_AGC_TARGET(& self)           -> i32 { unsafe { self.get_i32(GET::AGC_TARGET) } }
    pub fn get_DENOISE(& self)              -> bool { unsafe { self.get_bool(GET::DENOISE) } }
    pub fn get_DEREVERB(& self)             -> bool { unsafe { self.get_bool(GET::DEREVERB) } }
    pub fn get_DEREVERB_DECAY(& self)       -> f32 { unsafe { self.get_f32(GET::DEREVERB_DECAY) } }
    pub fn get_DEREVERB_LEVEL(& self)       -> f32 { unsafe { self.get_f32(GET::DEREVERB_LEVEL) } }
    pub fn get_ECHO_STATE(& self)           -> *mut SpeexEchoState {
        let arg = ptr::null_mut();
        unsafe { self.get_(GET::ECHO_STATE as c_int, arg as *mut c_void) };
        arg
    }
    pub fn get_ECHO_SUPPRESS(& self)        -> i32 { unsafe { self.get_i32(GET::ECHO_SUPPRESS) } }
    pub fn get_ECHO_SUPPRESS_ACTIVE(& self) -> i32 { unsafe { self.get_i32(GET::ECHO_SUPPRESS_ACTIVE) } }
    pub fn get_NOISE_PSD(& self)            -> i32 { unsafe { self.get_i32(GET::NOISE_PSD) } }
    pub fn get_NOISE_PSD_SIZE(& self)       -> i32 { unsafe { self.get_i32(GET::NOISE_PSD_SIZE) } }
    pub fn get_NOISE_SUPPRESS(& self)       -> i32 { unsafe { self.get_i32(GET::NOISE_SUPPRESS) } }
    pub fn get_PROB(& self)                 -> i32 { unsafe { self.get_i32(GET::PROB) } }
    pub fn get_PROB_CONTINUE(& self)        -> i32 { unsafe { self.get_i32(GET::PROB_CONTINUE) } }
    pub fn get_PROB_START(& self)           -> i32 { unsafe { self.get_i32(GET::PROB_START) } }
    pub fn get_PSD(& self)                  -> i32 { unsafe { self.get_i32(GET::PSD) } }
    pub fn get_PSD_SIZE(& self)             -> i32 { unsafe { self.get_i32(GET::PSD_SIZE) } }
    pub fn get_VAD(& self)                  -> bool { unsafe { self.get_bool(GET::VAD) } }


    pub fn set_AGC(&mut self, arg: bool)                         -> &Self { unsafe { self.set_bool(SET::AGC, arg) }; self }
    pub fn set_AGC_DECREMENT(&mut self, arg: i32)                -> &Self { unsafe { self.set_i32(SET::AGC_DECREMENT, arg) }; self }
    pub fn set_AGC_INCREMENT(&mut self, arg: i32)                -> &Self { unsafe { self.set_i32(SET::AGC_INCREMENT, arg) }; self }
    pub fn set_AGC_LEVEL(&mut self, arg: f32)                    -> &Self { unsafe { self.set_f32(SET::AGC_LEVEL, arg) }; self }
    pub fn set_AGC_MAX_GAIN(&mut self, arg: i32)                 -> &Self { unsafe { self.set_i32(SET::AGC_MAX_GAIN, arg) }; self }
    pub fn set_AGC_TARGET(&mut self, arg: i32)                   -> &Self { unsafe { self.set_i32(SET::AGC_TARGET, arg) }; self }
    pub fn set_DENOISE(&mut self, arg: bool)                     -> &Self { unsafe { self.set_bool(SET::DENOISE, arg) }; self }
    pub fn set_DEREVERB(&mut self, arg: bool)                    -> &Self { unsafe { self.set_bool(SET::DEREVERB, arg) }; self }
    pub fn set_DEREVERB_DECAY(&mut self, arg: f32)               -> &Self { unsafe { self.set_f32(SET::DEREVERB_DECAY, arg) }; self }
    pub fn set_DEREVERB_LEVEL(&mut self, arg: f32)               -> &Self { unsafe { self.set_f32(SET::DEREVERB_LEVEL, arg) }; self }
    pub fn set_ECHO_STATE(&mut self, arg: *const SpeexEchoState) -> &Self { unsafe { self.set_(SET::ECHO_STATE as c_int, arg as *mut c_void) }; self }
    pub fn set_ECHO_SUPPRESS(&mut self, arg: i32)                -> &Self { unsafe { self.set_i32(SET::ECHO_SUPPRESS, arg) }; self }
    pub fn set_ECHO_SUPPRESS_ACTIVE(&mut self, arg: i32)         -> &Self { unsafe { self.set_i32(SET::ECHO_SUPPRESS_ACTIVE, arg) }; self }
    pub fn set_NOISE_SUPPRESS(&mut self, arg: i32)               -> &Self { unsafe { self.set_i32(SET::NOISE_SUPPRESS, arg) }; self }
    pub fn set_PROB_CONTINUE(&mut self, arg: i32)                -> &Self { unsafe { self.set_i32(SET::PROB_CONTINUE, arg) }; self }
    pub fn set_PROB_START(&mut self, arg: i32)                   -> &Self { unsafe { self.set_i32(SET::PROB_START, arg) }; self }
    pub fn set_VAD(&mut self, arg: bool)                         -> &Self { unsafe { self.set_bool(SET::VAD, arg) }; self }
}

// impl Drop for SpeexPreProcess {
//     fn drop(&mut self) {
//         let lib = self.lib.unwrap();
//         match self.state {
//             None => (),
//             Some(state) => {
//                 // unsafe { lib.speex_preprocess_state_destroy(state) }
//             }
//         }
//     }
// }

