#![feature(saturating_neg)]

use anyhow::Result;
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access};

use crossbeam_channel::{Receiver,unbounded};

#[derive(argh::FromArgs)]
/// Record sound fragments from ALSA and play them back when silence is detected
struct Opt {
    /// ALSA device to record from
    #[argh(option, short='R', default="\"default\".to_string()")]
    record_device: String,

    /// ALSA device to play into
    #[argh(option, short='P', default="\"default\".to_string()")]
    playback_device: String,

    /// sample rate for recording and playing back.
    /// Format is always 1 channel, s16. Default 48000.
    #[argh(option, short='r', default="48000")]
    sample_rate: u32,

    /// threshold of maximum i16 sample value, default 250.
    /// Audio louder than that triggers ON mode
    #[argh(option, short='t', default="1000")]
    threshold: i16,

    /// number of samples per block
    #[argh(option, short='l', default="4800")]
    block_size: usize,

    /// number of silent blocks to wait before considering it "OFF".
    #[argh(option, short='h', default="5")]
    hysteresis: usize,
}

fn player(pcm: PCM, r: Receiver<Vec<i16>>) -> Result<()> {    
    let io = pcm.io_i16()?;
    
    let zb = [0; 4800];

    loop {
        let ret;

        match r.try_recv() {
            Ok(buf) => ret = io.writei(&buf[..]),
            Err(_) => ret = io.writei(&zb[..]),
        }
        
        match ret {
            //Ok(x) if x == 4800 => (),
            Ok(_x) => (), //eprintln!("write {}", _x),
            Err(e) => {
                pcm.try_recover(e, true)?;
                eprintln!("recovered");
            }
        }
        
    }
}

fn audio_level(buf:&[i16]) -> i16 {
    let mut x = 0;
    for b in buf {
        let mut b = *b;
        if b < 0 { b = b.saturating_neg(); }
        if x < b { x = b; }
    }
    x
}

fn main() -> Result<()> {
    let opt : Opt = argh::from_env();

    let (s, r) = unbounded();
    let (s2, r2) = unbounded();

    let pcm = PCM::new(&opt.record_device, Direction::input(), false)?;

    let hwp = HwParams::any(&pcm)?;
    hwp.set_channels(1)?;
    hwp.set_rate(opt.sample_rate, ValueOr::Nearest)?;
    hwp.set_format(Format::s16())?;
    hwp.set_access(Access::RWInterleaved)?;
    //hwp.set_buffer_size(12000)?;
    pcm.hw_params(&hwp)?;

    let pcm2 = PCM::new(&opt.playback_device, Direction::output(), false)?;

    let hwp2 = HwParams::any(&pcm)?;
    hwp2.set_channels(1)?;
    hwp2.set_rate(opt.sample_rate, ValueOr::Nearest)?;
    hwp2.set_format(Format::s16())?;
    hwp2.set_access(Access::RWInterleaved)?;
    //hwp.set_buffer_size(24000)?;
    pcm2.hw_params(&hwp2)?;

    let _player = std::thread::spawn(move || {
        if let Err(e) = player(pcm2, r) {
            eprintln!("{}", e);
        }
    });
    
    let io = pcm.io_i16()?;

    let mut buf = vec![0i16; opt.block_size];

    let mut state = false;
    let mut counter = 0;

    loop {
        let ret = io.readi(&mut buf[..])?;
        let buf = &buf[..ret];
        let lvl = audio_level(buf);
        //println!("{}", audio_level(buf));

        if lvl > opt.threshold {
            if !state {
                println!("ON");
            }
            state = true;
            counter = opt.hysteresis;
        } else {
            if counter == 0 {
                if state {
                    println!("OFF");
                }
                state = false;

                while let Ok(buf) = r2.try_recv() {
                    s.send(buf)?;
                }
            } else {
                counter -= 1;
            }
        }

        if state {
            s2.send(buf.to_vec())?;
        }
    }
}
