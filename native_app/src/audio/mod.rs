mod ambient;
mod car_sounds;
mod music;

use crate::audio::ambient::Ambient;
use crate::audio::car_sounds::CarSounds;
use crate::audio::music::Music;
use crate::gui::windows::settings::Settings;
use crate::uiworld::UiWorld;
use common::AudioKind;
use common::FastMap;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use egregoria::Egregoria;
use oddio::{Filter, Frames, FramesSignal, Gain, Handle, Mixer, Sample, Signal, Smoothed, Stop};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub struct GameAudio {
    music: Music,
    ambiant: Ambient,
    carsounds: CarSounds,
}

impl GameAudio {
    pub fn new(ctx: &mut AudioContext) -> Self {
        Self {
            music: Music::new(),
            ambiant: Ambient::new(ctx),
            carsounds: CarSounds::new(ctx),
        }
    }

    pub fn update(&mut self, goria: &Egregoria, uiworld: &mut UiWorld, ctx: &mut AudioContext) {
        self.music.update(ctx);
        self.ambiant.update(goria, uiworld);
        self.carsounds.update(goria, uiworld, ctx);
    }
}

type StoredAudio = Arc<Frames<[Sample; 2]>>;

// We allow dead_code because we need to keep OutputStream alive for it to work
#[allow(dead_code)]
pub struct AudioContext {
    stream: Option<cpal::Stream>,
    scene_handle: Option<Handle<Mixer<[Sample; 2]>>>,
    cache: FastMap<String, StoredAudio>,
}

static MUSIC_SHARED: AtomicU32 = AtomicU32::new(0);
static EFFECT_SHARED: AtomicU32 = AtomicU32::new(0);
static UI_SHARED: AtomicU32 = AtomicU32::new(0);

type ControlHandle<T> = Handle<Stop<GlobalGain<T>>>;
type Stereo = [Sample; 2];
type BaseSignal = FramesSignal<Stereo>;

impl AudioContext {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let sample_rate = device.default_output_config().unwrap().sample_rate();
        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let (scene_handle, scene) = oddio::split(Mixer::new());

        let stream = match device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let frames = oddio::frame_stereo(data);
                oddio::run(&scene, sample_rate.0, frames);
            },
            move |err| {
                eprintln!("{:?}", err);
            },
        ) {
            Ok(x) => x,
            Err(e) => {
                log::error!("Couldn't initialize audio because of {:?}", e);
                return Self {
                    stream: None,
                    scene_handle: None,
                    cache: Default::default(),
                };
            }
        };
        stream.play().unwrap();

        Self {
            stream: Some(stream),
            scene_handle: Some(scene_handle),
            cache: Default::default(),
        }
    }

    pub fn preload<'a>(&mut self, sounds: impl Iterator<Item = &'a str> + Send) {
        self.cache.extend(
            sounds
                .par_bridge()
                .flat_map(|x| Some(x.to_string()).zip(Self::decode(x)))
                .collect::<Vec<_>>()
                .into_iter(),
        );
    }

    pub fn g_volume(&self, kind: AudioKind) -> f32 {
        match kind {
            AudioKind::Music => f32::from_bits(MUSIC_SHARED.load(Ordering::Relaxed)),
            AudioKind::Effect => f32::from_bits(EFFECT_SHARED.load(Ordering::Relaxed)),
            AudioKind::Ui => f32::from_bits(UI_SHARED.load(Ordering::Relaxed)),
        }
    }

    fn decode(name: &str) -> Option<StoredAudio> {
        let p = format!("assets/sounds/{}.ogg", name);
        let buf = match std::fs::read(&p) {
            Ok(x) => x,
            Err(e) => {
                log::error!("Could not load sound {}: {}", name, e);
                return None;
            }
        };

        let mut decoder =
            lewton::inside_ogg::OggStreamReader::new(std::io::Cursor::new(buf)).unwrap();

        let mut samples = vec![];
        let mono = decoder.ident_hdr.audio_channels == 1;

        while let Some(packets) = decoder.read_dec_packet().expect("error decoding") {
            if mono {
                let mut it = packets.into_iter();
                let center = it.next().unwrap();
                samples.extend(center.into_iter().map(|x| {
                    let v = x as f32 / (i16::MAX as f32);
                    [v, v]
                }))
            } else {
                let mut it = packets.into_iter();
                let left = it.next().unwrap();
                let right = it.next().unwrap();
                samples.extend(
                    left.into_iter()
                        .zip(right)
                        .map(|(x, y)| [x as f32 / (i16::MAX as f32), y as f32 / (i16::MAX as f32)]),
                )
            }
        }

        log::info!(
            "decoding {}: {} sps|{} total samples|{} channels",
            &p,
            decoder.ident_hdr.audio_sample_rate,
            samples.len(),
            decoder.ident_hdr.audio_channels,
        );

        Some(Frames::from_slice(
            decoder.ident_hdr.audio_sample_rate,
            &samples,
        ))
    }

    fn get(cache: &mut FastMap<String, StoredAudio>, name: &str) -> Option<StoredAudio> {
        if let Some(v) = cache.get(name) {
            return Some(v.clone());
        }
        if let Some(decoded) = Self::decode(name) {
            cache.insert(name.to_string(), decoded.clone());
            return Some(decoded);
        }
        None
    }

    pub fn play(&mut self, name: &'static str, kind: AudioKind) {
        let vol = self.g_volume(kind);
        if let Some(ref mut h) = self.scene_handle {
            if let Some(x) = Self::get(&mut self.cache, name) {
                if let AudioKind::Music = kind {
                    log::error!("shouldn't play music with base play as it's not affected by global volume changes");
                }
                log::info!("playing {}", name);
                h.control().play(Gain::new(FramesSignal::new(x, 0.0), vol));
            }
        }
    }

    pub fn play_with_control<S: 'static>(
        &mut self,
        name: &'static str,
        transform: impl FnOnce(StoredAudio) -> S,
        kind: AudioKind,
    ) -> Option<ControlHandle<S>>
    where
        S: Signal<Frame = [Sample; 2]> + Send,
    {
        if let Some(ref mut h) = self.scene_handle {
            if let Some(x) = Self::get(&mut self.cache, name) {
                let test = GlobalGain {
                    volume: RefCell::new(Smoothed::new(1.0)),
                    kind,
                    inner: transform(x),
                };
                let hand = h.control().play(test);
                return Some(hand);
            }
        }
        None
    }

    pub fn set_settings(&mut self, settings: &Settings) {
        let ui_volume = (settings.ui_volume_percent / 100.0).powi(2);
        if (f32::from_bits(UI_SHARED.load(Ordering::Relaxed)) - ui_volume).abs() > f32::EPSILON {
            UI_SHARED.store(ui_volume.to_bits(), Ordering::Relaxed);
        }

        let music_volume = (settings.music_volume_percent / 100.0).powi(2);
        if (f32::from_bits(MUSIC_SHARED.load(Ordering::Relaxed)) - music_volume).abs()
            > f32::EPSILON
        {
            MUSIC_SHARED.store(music_volume.to_bits(), Ordering::Relaxed);
        }

        let effect_volume = (settings.effects_volume_percent / 100.0).powi(2);
        if (f32::from_bits(EFFECT_SHARED.load(Ordering::Relaxed)) - effect_volume).abs()
            > f32::EPSILON
        {
            EFFECT_SHARED.store(effect_volume.to_bits(), Ordering::Relaxed);
        }
    }
}

pub struct GlobalGain<T: ?Sized> {
    volume: RefCell<Smoothed<f32>>,
    kind: AudioKind,
    inner: T,
}

impl<T: Signal<Frame = [Sample; 2]>> Signal for GlobalGain<T> {
    type Frame = [Sample; 2];

    fn sample(&self, interval: f32, out: &mut [Self::Frame]) {
        fn upd(x: &AtomicU32, gain: &mut std::cell::RefMut<Smoothed<f32>>) {
            let shared = f32::from_bits(x.load(Ordering::Relaxed));
            if gain.get() != shared {
                gain.set(shared);
            }
        }

        let mut gain = self.volume.borrow_mut();
        match self.kind {
            AudioKind::Music => {
                upd(&MUSIC_SHARED, &mut gain);
            }
            AudioKind::Effect => {
                upd(&EFFECT_SHARED, &mut gain);
            }
            AudioKind::Ui => {
                upd(&UI_SHARED, &mut gain);
            }
        };

        if gain.get() == 0.0 {
            out.fill([0.0; 2]);
            return;
        }

        self.inner.sample(interval, out);
        for x in out {
            let g = gain.get();
            x[0] *= g;
            x[1] *= g;
            gain.advance(interval / 0.1);
        }
    }

    fn remaining(&self) -> f32 {
        self.inner.remaining()
    }

    fn handle_dropped(&self) {
        self.inner.handle_dropped()
    }
}

impl<T> Filter for GlobalGain<T> {
    type Inner = T;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }
}

pub struct FadeIn<T: ?Sized> {
    fadetime: f32,
    advance: RefCell<f32>,
    inner: T,
}

impl<T> FadeIn<T> {
    pub fn new(signal: T, fadetime: f32) -> Self {
        Self {
            fadetime,
            advance: RefCell::new(0.0),
            inner: signal,
        }
    }
}

impl<T: Signal<Frame = [Sample; 2]>> Signal for FadeIn<T> {
    type Frame = [Sample; 2];

    fn sample(&self, interval: f32, out: &mut [Self::Frame]) {
        self.inner.sample(interval, out);

        let mut advance = self.advance.borrow_mut();
        if *advance >= 1.0 {
            return;
        }

        for x in out {
            x[0] *= *advance;
            x[1] *= *advance;
            *advance += interval / self.fadetime;
        }
    }

    fn remaining(&self) -> f32 {
        self.inner.remaining()
    }

    fn handle_dropped(&self) {
        self.inner.handle_dropped()
    }
}

impl<T> Filter for FadeIn<T> {
    type Inner = T;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }
}
