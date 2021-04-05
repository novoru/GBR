use crate::core::io::Io;
use std::sync::{Arc, Mutex};
use cpal::{Stream, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Copy, Clone, Debug)]
struct Channel {
    no:                     u8,     // number of channel (for debug)
    freq:                   f32,
    amplitude:              f32,
    sample_rate:            f32,

    duration:               i32,
    length:                 u32,

    envelope_time:          f32,
    envelope_samples:       f32,
    envelope_volume:        u32,
    envelope_steps:         u32,
    envelope_steps_init:    u32,
    envelope_increasing:    bool,
}

impl Channel {
    pub fn new(no: u8) -> Self {
        println!("sampling rate: {}", get_sample_rate());
        Channel {
            no:                     no,
            freq:                   0f32,
            amplitude:              1f32,
            sample_rate:            get_sample_rate(),
            duration:               0i32,
            length:                 0u32,
            envelope_time:          0f32,
            envelope_samples:       0f32,
            envelope_volume:        0u32,
            envelope_steps:         0u32,
            envelope_steps_init:    0u32,
            envelope_increasing:    true,
        }
    }

    pub fn reset(&mut self) {
        self.amplitude = 1f32;
        self.envelope_time = 0f32;
    }

    pub fn update_envelope(&mut self) {
        if self.envelope_samples > 0f32 {
            self.envelope_time += 1f32 / self.sample_rate;
            if self.envelope_steps > 0 && self.envelope_time >= self.envelope_samples {
                self.envelope_time = 0f32;
                self.envelope_steps -= 1;
                if self.envelope_steps == 0 {
                    self.amplitude = 0f32;
                } else if self.envelope_increasing {
                    self.amplitude = 1f32 - (self.envelope_steps as f32)/(self.envelope_steps_init as f32);
                } else {
                    self.amplitude = (self.envelope_steps as f32)/(self.envelope_steps_init as f32);
                }
            }
        }
    }

    pub fn update_sweep(&mut self) {
    }

    pub fn should_play(&mut self) -> bool {
        (self.duration == -1 || self.duration > 0) &&
         self.envelope_steps_init > 0
    }
}

pub struct Apu {
    // Sound Channel 1
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    channel1:    Arc<Mutex<Channel>>,
    stream1:    Stream,
    
    // Sound Channel 2
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    channel2:    Arc<Mutex<Channel>>,
    stream2:    Stream,

    // Sound Channel 3
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    wavepattern_ram: [u8; 0x10],
    
    // Sound Channel 4
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,

    // Sound Control Registers
    nr50: u8,
    nr51: u8,
    nr52: u8,
}

impl Apu {
    pub fn new() -> Self {
        let channel1 = Arc::new(Mutex::new(Channel::new(1)));
        let stream1 = get_stream(channel1.clone());
        stream1.play().unwrap();
        
        let channel2 = Arc::new(Mutex::new(Channel::new(2)));
        let stream2 = get_stream(channel2.clone());
        stream2.play().unwrap();

        Apu {
         nr10:      0x80,
         nr11:      0xBF,
         nr12:      0xF3,
         nr13:      0x00,
         nr14:      0xBF,
         channel1:   channel1,
         stream1:   stream1,

         nr21:      0x3F,
         nr22:      0x00,
         nr23:      0x00,
         nr24:      0xBF,
         channel2:   channel2,
         stream2:   stream2,
         
         nr30:  0x7F,
         nr31:  0xFF,
         nr32:  0x9F,
         nr33:  0xBF,
         nr34:  0x00,
         wavepattern_ram:    [0; 0x10],
         
         nr41:  0xFF,
         nr42:  0x00,
         nr43:  0x00,
         nr44:  0x00,
         
         nr50:  0x77,
         nr51:  0xF3,
         nr52:  0xF1,
        }
    }
}

impl Io for Apu {
    fn read8(&self, addr: usize) -> u8 {
        match addr {
            0xFF10              =>  self.nr10,
            0xFF11              =>  self.nr11,
            0xFF12              =>  self.nr12,
            0xFF13              =>  self.nr13,
            0xFF14              =>  self.nr14,
            0xFF16              =>  self.nr21,
            0xFF17              =>  self.nr22,
            0xFF18              =>  self.nr23,
            0xFF19              =>  self.nr24,
            0xFF1A              =>  self.nr30,
            0xFF1B              =>  self.nr31,
            0xFF1C              =>  self.nr32,
            0xFF1D              =>  self.nr33,
            0xFF1E              =>  self.nr34,
            0xFF30 ..= 0xFF3F   =>  self.wavepattern_ram[addr-0xFF30],
            0xFF20              =>  self.nr41,
            0xFF21              =>  self.nr42,
            0xFF22              =>  self.nr43,
            0xFF23              =>  self.nr44,
            0xFF24              =>  self.nr50,
            0xFF25              =>  self.nr51,
            0xFF26              =>  self.nr52,
            _                   =>  panic!("can't read from: {:04x}", addr),
        }
    }

    fn write8(&mut self, addr: usize, data: u8) {
        match addr {
            0xFF10              =>  self.nr10 = data,
            0xFF11              =>  {
                self.nr11 = data;
                if let Ok(mut channel) = self.channel1.lock() {
                    channel.length = (self.nr11 & 0x3F) as u32;
                };
            },
            0xFF12              =>  {
                self.nr12 = data;
                if let Ok(mut channel) = self.channel1.lock() {
                    channel.envelope_volume     = ((self.nr12 & 0xF0) >> 4) as u32;
                    channel.envelope_samples    = ((self.nr12 & 0x07) as f32) / 64f32;
                    channel.envelope_increasing = (((self.nr12 & 0x08) >> 3) == 1) as bool;
                };
            },
            0xFF13              =>  {
                self.nr13 = data;
                let freq = (131072 / (2048 - ((self.nr13 as u32) + (((self.nr14 & 0b111) as u32) << 8)))) as f32;
                if let Ok(mut channel) = self.channel1.lock() {
                    channel.freq = freq;
                };
            },
            0xFF14              =>  {
                self.nr14 = data;
                let freq = (131072 / (2048 - ((self.nr13 as u32) + (((self.nr14 & 0b111) as u32) << 8)))) as f32;
                if let Ok(mut channel) = self.channel1.lock() {
                    channel.freq = freq;
                    if self.nr14 & 0x80 != 0{
                        if channel.length == 0 {
                            channel.length = 64;
                        }
                        let mut duration = -1;
                        if self.nr14 & 0x40 != 0 {
                            duration = ((channel.length as f32) * (1f32/64f32)) as i32 * channel.sample_rate as i32;
                        }
                        channel.duration = duration;
                        channel.reset();
                        channel.envelope_steps = channel.envelope_volume;
                        channel.envelope_steps_init = channel.envelope_volume;
                    }
                };
            },
            0xFF16              =>  self.nr21 = data,
            0xFF17              =>  {
                self.nr22 = data;
                if let Ok(mut channel) = self.channel2.lock() {
                    channel.envelope_volume     = ((self.nr22 & 0xF0) >> 4) as u32;
                    channel.envelope_samples    = ((self.nr22 & 0x07) as f32) / 64f32;
                    channel.envelope_increasing = (((self.nr22 & 0x08) >> 3) == 1) as bool;
                };
            },
            0xFF18              =>  {
                self.nr23 = data;
                let freq = (131072 / (2048 - ((self.nr23 as u32) + (((self.nr24 & 0b111) as u32) << 8)))) as f32;
                if let Ok(mut channel) = self.channel2.lock() {
                    channel.freq = freq;
                };
            },
            0xFF19              =>  {
                self.nr24 = data;
                let freq = (131072 / (2048 - ((self.nr23 as u32) + (((self.nr24 & 0b111) as u32) << 8)))) as f32;
                if let Ok(mut channel) = self.channel2.lock() {
                    channel.freq = freq;
                    if self.nr24 & 0x80 != 0 {
                        if channel.length == 0 {
                            channel.length = 64;
                        }
                        let mut duration = -1;
                        if self.nr24 & 0x40 != 0 {
                            duration = ((channel.length as f32) * (1f32/64f32)) as i32 * channel.sample_rate as i32;
                        }
                        channel.duration = duration;
                        channel.reset();
                        channel.envelope_steps = channel.envelope_volume;
                        channel.envelope_steps_init = channel.envelope_volume;
                    }
                };
            },
            0xFF1A              =>  self.nr30 = data,
            0xFF1B              =>  self.nr31 = data,
            0xFF1C              =>  self.nr32 = data,
            0xFF1D              =>  self.nr33 = data,
            0xFF1E              =>  self.nr34 = data,
            0xFF30 ..= 0xFF3F   =>  self.wavepattern_ram[addr-0xFF30] = data,
            0xFF20              =>  self.nr41 = data,
            0xFF21              =>  self.nr42 = data,
            0xFF22              =>  self.nr43 = data,
            0xFF23              =>  self.nr44 = data,
            0xFF24              =>  self.nr50 = data,
            0xFF25              =>  self.nr51 = data,
            0xFF26              =>  self.nr52 = data,
            _       => panic!("can't write to: {:04x}", addr),
        }
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn get_stream(channel_arc: Arc<Mutex<Channel>>) -> Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();
    let channels = config.channels as usize;
    let sample_rate = config.sample_rate.0 as f32;
    let mut sample_clock = 0f32;
    let mut prev = 0f32;

    let mut call_back = move || {
        sample_clock = (sample_clock + 1f32) % sample_rate;
        let mut output = prev;
        
        if let Ok(mut channel) = channel_arc.lock() {
            if channel.should_play() {
                output = channel.amplitude * ((sample_clock * channel.freq * 2.0 * std::f32::consts::PI / sample_rate)
                            .sin().ceil()) / 20.0;
                prev = output;
                if channel.duration > 0 {
                    channel.duration -= 1;
                }
            }
            channel.update_envelope();
            channel.update_sweep();
        }

        output
    };

    match sample_format {
        SampleFormat::F32 => device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut call_back)
            },
            err_fn
        ),
        _   => panic!(),
    }.unwrap()
}

fn get_sample_rate() -> f32 {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let config: cpal::StreamConfig = supported_config.into();
    config.sample_rate.0 as f32
}