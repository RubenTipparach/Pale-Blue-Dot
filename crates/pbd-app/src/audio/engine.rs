//! kira, held for the game (`audio-and-ambience` design sections 1, 2, 8 and
//! 9): the manager on a sound device or on the mock backend, the buses, the
//! reverb send, the bus meters, the WAV tap, and the bank of decoded cues.

use kira::{
    AudioManager, AudioManagerSettings, Decibels, Frame, Mix, Tween,
    backend::{
        cpal::{CpalBackend, CpalBackendSettings},
        mock::{MockBackend, MockBackendSettings},
    },
    effect::{
        Effect, EffectBuilder,
        filter::{FilterBuilder, FilterHandle},
        reverb::ReverbBuilder,
    },
    info::Info,
    sound::static_sound::StaticSoundData,
    track::{MainTrackBuilder, SendTrackBuilder, SendTrackHandle, TrackBuilder, TrackHandle},
};
use pbd_core::audio::CueBook;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, Ordering},
};
use std::time::{Duration, Instant};

/// The sample rate the mock backend runs at, and the files are made at.
pub const MOCK_RATE: u32 = 48_000;

/// Where the audio files live.
pub fn audio_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/audio"))
}

/// Every cue decoded whole, by name, one entry per take.
pub struct Bank {
    pub sounds: HashMap<String, Vec<StaticSoundData>>,
    /// Decoded size, bytes (stereo f32 frames).
    pub bytes: usize,
    pub files: usize,
    pub seconds: f32,
    /// Files that would not decode.
    pub failed: Vec<String>,
}

/// Decode every cue in `book` from `dir`, on a few threads.
pub fn decode_bank(dir: &Path, book: &CueBook) -> Bank {
    let started = Instant::now();
    let jobs: Vec<(String, usize, PathBuf)> = book
        .cues
        .iter()
        .filter(|(_, cue)| !cue.stream)
        .flat_map(|(name, cue)| {
            cue.files(name)
                .into_iter()
                .enumerate()
                .map(move |(take, file)| (name.clone(), take, dir.join(file)))
        })
        .collect();
    let threads = std::thread::available_parallelism()
        .map_or(2, |n| n.get())
        .clamp(1, 4);
    let chunk = jobs.len().div_ceil(threads).max(1);
    let decoded: Vec<(String, usize, Result<StaticSoundData, String>)> =
        std::thread::scope(|scope| {
            let handles: Vec<_> = jobs
                .chunks(chunk)
                .map(|part| {
                    scope.spawn(move || {
                        part.iter()
                            .map(|(name, take, path)| {
                                let data = StaticSoundData::from_file(path)
                                    .map_err(|e| format!("{}: {e}", path.display()));
                                (name.clone(), *take, data)
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap_or_default())
                .collect()
        });
    let mut sounds: HashMap<String, Vec<(usize, StaticSoundData)>> = HashMap::new();
    let mut failed = Vec::new();
    let mut bytes = 0;
    let files = decoded.len();
    for (name, take, data) in decoded {
        match data {
            Ok(data) => {
                bytes += data.num_frames() * 8;
                sounds.entry(name).or_default().push((take, data));
            }
            Err(why) => failed.push(why),
        }
    }
    let sounds = sounds
        .into_iter()
        .map(|(name, mut takes)| {
            takes.sort_by_key(|(take, _)| *take);
            (name, takes.into_iter().map(|(_, d)| d).collect())
        })
        .collect();
    Bank {
        sounds,
        bytes,
        files,
        seconds: started.elapsed().as_secs_f32(),
        failed,
    }
}

/// A bus meter: the peak since it was last read, and the last block's RMS,
/// as `f32` bits (design section 9, item 4).
#[derive(Default)]
pub struct MeterShared {
    peak: AtomicU32,
    rms: AtomicU32,
}

impl MeterShared {
    /// Peak since the last read, and the latest RMS, both linear.
    pub fn take(&self) -> (f32, f32) {
        (
            f32::from_bits(self.peak.swap(0, Ordering::Relaxed)),
            f32::from_bits(self.rms.load(Ordering::Relaxed)),
        )
    }
}

struct MeterEffect(Arc<MeterShared>);

impl Effect for MeterEffect {
    fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
        let mut peak = 0f32;
        let mut sum = 0f32;
        for frame in input.iter() {
            peak = peak.max(frame.left.abs()).max(frame.right.abs());
            sum += frame.left * frame.left + frame.right * frame.right;
        }
        let rms = (sum / (2 * input.len().max(1)) as f32).sqrt();
        // Positive floats order as their bits do.
        self.0.peak.fetch_max(peak.to_bits(), Ordering::Relaxed);
        self.0.rms.store(rms.to_bits(), Ordering::Relaxed);
    }
}

struct Meter(Arc<MeterShared>);

impl EffectBuilder for Meter {
    type Handle = ();
    fn build(self) -> (Box<dyn Effect>, ()) {
        (Box::new(MeterEffect(self.0)), ())
    }
}

/// The mix as it leaves `main`, interleaved stereo, for `--audio-wav`.
pub type Tapped = Arc<Mutex<Vec<f32>>>;

struct TapEffect(Tapped);

impl Effect for TapEffect {
    fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info) {
        // Never block the audio thread: a busy writer loses a block, which
        // only happens on a real device (the mock runs on the main thread).
        if let Ok(mut out) = self.0.try_lock() {
            for frame in input.iter() {
                out.push(frame.left);
                out.push(frame.right);
            }
        }
    }
}

struct Tap(Tapped);

impl EffectBuilder for Tap {
    type Handle = ();
    fn build(self) -> (Box<dyn Effect>, ()) {
        (Box::new(TapEffect(self.0)), ())
    }
}

/// Which backend to open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// The system's sound device.
    Device,
    /// No device: the mixer advanced by hand, this many frames each game
    /// frame (`--capture`).
    Mock { frames_per_step: usize },
}

enum Manager {
    Device(AudioManager<CpalBackend>),
    Mock(AudioManager<MockBackend>),
}

macro_rules! with {
    ($manager:expr, $m:ident => $body:expr) => {
        match $manager {
            Manager::Device($m) => $body,
            Manager::Mock($m) => $body,
        }
    };
}

/// The buses (design section 2).
pub struct Engine {
    manager: Manager,
    pub world: TrackHandle,
    pub world_filter: FilterHandle,
    pub reverb: SendTrackHandle,
    pub effects: TrackHandle,
    pub ambience: TrackHandle,
    pub ambience_filter: FilterHandle,
    pub music: TrackHandle,
    pub menus: TrackHandle,
    pub meters: Vec<(&'static str, Arc<MeterShared>)>,
    pub tap: Option<Tapped>,
    pub sample_rate: u32,
}

impl Engine {
    /// Open the manager and lay out the buses. `tap` writes the main mix
    /// for `--audio-wav`.
    pub fn open(backend: Backend, tap: bool) -> Result<Self, String> {
        let mut meters = Vec::new();
        let mut meter = |name: &'static str| {
            let shared = Arc::new(MeterShared::default());
            meters.push((name, shared.clone()));
            Meter(shared)
        };
        let tapped: Option<Tapped> = tap.then(|| Arc::new(Mutex::new(Vec::new())));
        // A gentle limiter first, so a near strike over a storm does not
        // clip the sum; then the meter and the tap see what leaves.
        let mut main = MainTrackBuilder::new()
            .with_effect(
                kira::effect::compressor::CompressorBuilder::new()
                    .threshold(-4.0)
                    .ratio(8.0)
                    .attack_duration(Duration::from_millis(2))
                    .release_duration(Duration::from_millis(120)),
            )
            .with_effect(meter("main"));
        if let Some(tapped) = &tapped {
            main.add_effect(Tap(tapped.clone()));
        }
        let capacities = kira::Capacities {
            sub_track_capacity: 256,
            ..Default::default()
        };
        let (mut manager, sample_rate) = match backend {
            Backend::Device => {
                // cpal can panic deep in a platform layer on a machine with
                // no sound stack; that is "no device" too.
                let opened = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    AudioManager::<CpalBackend>::new(AudioManagerSettings {
                        capacities,
                        main_track_builder: main,
                        internal_buffer_size: 128,
                        backend_settings: CpalBackendSettings::default(),
                    })
                }))
                .map_err(|_| "the sound device panicked on open".to_string())?
                .map_err(|e| e.to_string())?;
                (Manager::Device(opened), 0)
            }
            Backend::Mock { frames_per_step } => {
                let opened = AudioManager::<MockBackend>::new(AudioManagerSettings {
                    capacities,
                    main_track_builder: main,
                    internal_buffer_size: frames_per_step.max(1),
                    backend_settings: MockBackendSettings {
                        sample_rate: MOCK_RATE,
                    },
                })
                .map_err(|_| "the mock backend did not open".to_string())?;
                (Manager::Mock(opened), MOCK_RATE)
            }
        };
        let limit = |e: kira::ResourceLimitReached| e.to_string();
        let reverb = with!(&mut manager, m => m.add_send_track(
            SendTrackBuilder::new().with_effect(
                ReverbBuilder::new().feedback(0.82).damping(0.6).mix(Mix::WET),
            ),
        ))
        .map_err(limit)?;
        let mut world_builder = TrackBuilder::new()
            .with_send(reverb.id(), Decibels(-60.0))
            .with_effect(meter("world"));
        let world_filter = world_builder.add_effect(FilterBuilder::new().cutoff(20_000.0));
        let mut world = with!(&mut manager, m => m.add_sub_track(world_builder)).map_err(limit)?;
        let effects = world
            .add_sub_track(TrackBuilder::new().with_effect(meter("effects")))
            .map_err(limit)?;
        let mut ambience_builder = TrackBuilder::new().with_effect(meter("ambience"));
        let ambience_filter = ambience_builder.add_effect(FilterBuilder::new().cutoff(20_000.0));
        let ambience = world.add_sub_track(ambience_builder).map_err(limit)?;
        let music = with!(&mut manager, m => m.add_sub_track(
            TrackBuilder::new().with_effect(meter("music"))
        ))
        .map_err(limit)?;
        let menus = with!(&mut manager, m => m.add_sub_track(
            TrackBuilder::new().with_effect(meter("menus"))
        ))
        .map_err(limit)?;
        Ok(Self {
            manager,
            world,
            world_filter,
            reverb,
            effects,
            ambience,
            ambience_filter,
            music,
            menus,
            meters,
            tap: tapped,
            sample_rate,
        })
    }

    pub fn is_mock(&self) -> bool {
        matches!(self.manager, Manager::Mock(_))
    }

    pub fn set_master(&mut self, db: f32, tween: Tween) {
        with!(&mut self.manager, m => m.main_track().set_volume(Decibels(db), tween));
    }

    /// The mock backend: mix one game frame's samples now.
    pub fn advance(&mut self) {
        if let Manager::Mock(m) = &mut self.manager {
            m.backend_mut().on_start_processing();
            m.backend_mut().process();
        }
    }

    /// The device's audio thread load, 0..1, when it has a new reading.
    pub fn cpu_usage(&mut self) -> Option<f32> {
        match &mut self.manager {
            Manager::Device(m) => m.backend_mut().pop_cpu_usage(),
            Manager::Mock(_) => None,
        }
    }

    /// A stream error the device reported (and recovered from), if any.
    pub fn device_error(&mut self) -> Option<String> {
        match &mut self.manager {
            Manager::Device(m) => m.backend_mut().pop_error().map(|e| e.to_string()),
            Manager::Mock(_) => None,
        }
    }
}

/// A tween of `seconds`.
pub fn tween(seconds: f32) -> Tween {
    Tween {
        duration: Duration::from_secs_f32(seconds.max(0.0)),
        ..Default::default()
    }
}

/// A 16-bit stereo WAV written as it grows (`--audio-wav`): the header's
/// sizes are patched after every append, so a run killed mid-way leaves a
/// playable file.
pub struct WavWriter {
    file: std::fs::File,
    samples: u32,
}

impl WavWriter {
    pub fn create(path: &Path, rate: u32) -> std::io::Result<Self> {
        use std::io::Write;
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)?;
        }
        let mut file = std::fs::File::create(path)?;
        let mut header = Vec::with_capacity(44);
        header.extend_from_slice(b"RIFF");
        header.extend_from_slice(&36u32.to_le_bytes());
        header.extend_from_slice(b"WAVEfmt ");
        header.extend_from_slice(&16u32.to_le_bytes());
        header.extend_from_slice(&1u16.to_le_bytes());
        header.extend_from_slice(&2u16.to_le_bytes());
        header.extend_from_slice(&rate.to_le_bytes());
        header.extend_from_slice(&(rate * 4).to_le_bytes());
        header.extend_from_slice(&4u16.to_le_bytes());
        header.extend_from_slice(&16u16.to_le_bytes());
        header.extend_from_slice(b"data");
        header.extend_from_slice(&0u32.to_le_bytes());
        file.write_all(&header)?;
        Ok(Self { file, samples: 0 })
    }

    pub fn append(&mut self, samples: &[f32]) -> std::io::Result<()> {
        use std::io::{Seek, SeekFrom, Write};
        if samples.is_empty() {
            return Ok(());
        }
        let bytes: Vec<u8> = samples
            .iter()
            .flat_map(|s| ((s.clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes())
            .collect();
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&bytes)?;
        self.samples += samples.len() as u32;
        let data = self.samples * 2;
        self.file.seek(SeekFrom::Start(4))?;
        self.file.write_all(&(36 + data).to_le_bytes())?;
        self.file.seek(SeekFrom::Start(40))?;
        self.file.write_all(&data.to_le_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kira::sound::streaming::StreamingSoundData;

    fn mock() -> Engine {
        Engine::open(
            Backend::Mock {
                frames_per_step: 800,
            },
            true,
        )
        .expect("mock opens")
    }

    fn run(engine: &mut Engine, frames: usize) {
        for _ in 0..frames {
            engine.advance();
        }
    }

    fn meter(engine: &Engine, name: &str) -> f32 {
        engine
            .meters
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, m)| m.take().0)
            .unwrap()
    }

    /// A cue decodes (the `vorbis` codec is built in) and is heard on the
    /// effects bus, through `world`, at `main`.
    #[test]
    fn a_cue_plays_through_the_buses() {
        let mut engine = mock();
        let data = StaticSoundData::from_file(audio_dir().join("sfx/break_stone.ogg"))
            .expect("an Ogg Vorbis cue decodes");
        engine.effects.play(data).unwrap();
        run(&mut engine, 10);
        assert!(meter(&engine, "effects") > 0.01);
        assert!(meter(&engine, "main") > 0.01);
        let tapped = engine.tap.as_ref().unwrap().lock().unwrap().len();
        assert_eq!(tapped, 10 * 800 * 2, "the tap holds every frame mixed");
    }

    /// A bed streams on the ambience bus.
    #[test]
    fn a_bed_streams_on_the_ambience_bus() {
        let mut engine = mock();
        let data = StreamingSoundData::from_file(audio_dir().join("ambience/amb_wind_low.ogg"))
            .expect("a bed opens");
        let mut sound = engine
            .ambience
            .play(
                data.loop_region(..)
                    .start_position(7.6)
                    .volume(Decibels::SILENCE),
            )
            .unwrap();
        sound.set_volume(Decibels(-6.0), tween(0.25));
        // The stream's decoder runs on its own thread: give it a moment.
        let mut heard = 0.0f32;
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(10));
            run(&mut engine, 2);
            heard = heard.max(meter(&engine, "ambience"));
        }
        assert!(heard > 0.001, "{heard}");
    }
}
