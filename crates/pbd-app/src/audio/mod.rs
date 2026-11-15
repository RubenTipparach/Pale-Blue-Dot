//! Sound (`audio-and-ambience`): kira on the sound device, cues placed on the
//! planet in the camera's body-local frame, the ambience beds, thunder, the
//! fires, the menus, the player's five volumes, and the instruments that let
//! a session that cannot listen check all of it (`--audio-log`,
//! `--audio-wav`, `--no-audio`).
//!
//! Anything may ask for a sound through [`Sfx`]; the mixer, which owns kira,
//! runs once a frame in `PostUpdate` and turns the asks into voices. The
//! rules (gain, cutoff, delay, beds, thunder, the voice cap) are
//! `pbd_core::audio`'s; this module only applies them.

pub mod engine;
pub mod hooks;

use crate::settings::{PlayerSettings, Volumes};
use bevy::prelude::*;
use engine::{Backend, Bank, Engine, WavWriter, tween};
use kira::{
    Decibels, Panning, PlaybackRate, StartTime,
    sound::{
        FromFileError, PlaybackState,
        static_sound::StaticSoundHandle,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
    track::{TrackBuilder, TrackHandle},
};
use pbd_core::audio::{
    Admit, Ambience, Bed, BedMix, Bus, CueBook, MixConfig, admit, air_cutoff_hz, amplitude,
    decibels, delay_s, distance_gain, pan,
};
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

/// How this run sounds, from its flags.
#[derive(Clone, Debug, Default)]
pub struct AudioLaunch {
    /// `--no-audio`: no device is opened and nothing is mixed.
    pub off: bool,
    /// `--audio-log`: every voice and bed change, and the buses each second.
    pub log: bool,
    /// `--audio-wav PATH`: the main mix written to a file.
    pub wav: Option<PathBuf>,
    /// A fixed-step `--capture`: the mock backend, one frame's samples per
    /// frame, fixed dice, so the WAV is the same every run.
    pub capture: bool,
    /// `PBD_TAKE_SOUND=1`, a capture or a benchmark: default volumes.
    pub default_volumes: bool,
}

pub struct AudioPlugin(pub AudioLaunch);

/// Where a sound is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Where {
    /// Unplaced: the menus bus.
    Menu,
    /// At a body-local point.
    At(Vec3),
}

/// A loop's identity: kept alive while asked for under the same key.
pub type LoopKey = (&'static str, u64);

#[derive(Clone, Debug)]
struct Request {
    cue: &'static str,
    at: Where,
    db: f32,
    rate: f32,
    /// How long ago the cause happened (a strike's flash), taken off the
    /// travel delay.
    ago_s: f32,
    key: Option<LoopKey>,
}

/// Asks for sounds this frame. A one-shot plays once; a loop asked for
/// under a key keeps playing while it is asked for every frame.
#[derive(Resource, Default)]
pub struct Sfx {
    requests: Vec<Request>,
}

impl Sfx {
    pub fn play(&mut self, cue: &'static str, at: Where) {
        self.play_with(cue, at, 0.0, 1.0);
    }

    /// With a level offset and a rate multiplier.
    pub fn play_with(&mut self, cue: &'static str, at: Where, db: f32, rate: f32) {
        self.requests.push(Request {
            cue,
            at,
            db,
            rate,
            ago_s: 0.0,
            key: None,
        });
    }

    /// A sound whose cause happened `ago_s` seconds ago: it arrives that much
    /// sooner (thunder after its flash).
    pub fn play_since(&mut self, cue: &'static str, at: Where, db: f32, ago_s: f32) {
        self.requests.push(Request {
            cue,
            at,
            db,
            rate: 1.0,
            ago_s,
            key: None,
        });
    }

    /// Keep a loop playing this frame.
    pub fn keep(&mut self, key: LoopKey, cue: &'static str, at: Where, db: f32, rate: f32) {
        self.requests.push(Request {
            cue,
            at,
            db,
            rate,
            ago_s: 0.0,
            key: Some(key),
        });
    }
}

/// Where the ears are this frame: the active camera, body-local.
#[derive(Resource, Clone, Copy, Debug)]
pub struct Listener {
    pub eye: Vec3,
    /// Body-local to the camera's own axes.
    pub inverse: Quat,
    pub forward: Vec3,
    pub valid: bool,
}

impl Default for Listener {
    fn default() -> Self {
        Self {
            eye: Vec3::ZERO,
            inverse: Quat::IDENTITY,
            forward: Vec3::NEG_Z,
            valid: false,
        }
    }
}

/// What the world's bus is doing, from the screens and the beds' read
/// (`hooks`): under water, shelter, cave depth, a pause or a menu over play.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct Environment {
    pub underwater: bool,
    pub sheltered: bool,
    /// 0 outdoors, 1 deep underground.
    pub cave: f32,
    /// `air_density_scale` at the eye.
    pub air: f32,
    /// A screen that stops the world (pause, settings, saves).
    pub paused: bool,
    /// A screen over a running world (map, pack, a station).
    pub ducked: bool,
    /// The beds' targets, from the last read.
    pub beds: Option<BedMix>,
}

/// Whether the sound bank is decoded: the loading screen waits on it.
#[derive(Resource, Clone, Default)]
pub struct SoundsReady(pub Arc<AtomicBool>);

impl SoundsReady {
    pub fn ready(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// The data, read at startup.
#[derive(Resource, Clone)]
pub struct AudioData {
    pub cues: CueBook,
    pub mix: MixConfig,
    pub ambience: Ambience,
}

fn config_path(name: &str) -> PathBuf {
    crate::config::config_dir().join(format!("audio/{name}.ron"))
}

/// Read `assets/config/audio/<name>.ron`. A file that fails to parse is a
/// startup error with its path, as every config is.
fn load<T: serde::de::DeserializeOwned + Default>(name: &str) -> T {
    let path = config_path(name);
    match std::fs::read_to_string(&path) {
        Ok(text) => ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
        Err(_) => {
            warn!("{} not found; using built-in defaults", path.display());
            T::default()
        }
    }
}

impl AudioData {
    pub fn load() -> Self {
        let data = Self {
            cues: load("cues"),
            mix: load("mix"),
            ambience: load("ambience"),
        };
        let mut problems = data.cues.problems();
        problems.extend(data.mix.problems());
        assert!(
            problems.is_empty(),
            "assets/config/audio: {}",
            problems.join("; ")
        );
        data
    }
}

/// A voice's sound: decoded at load, or streamed from disk.
enum VoiceSound {
    Static(StaticSoundHandle),
    Stream(StreamingSoundHandle<FromFileError>),
}

macro_rules! both {
    ($sound:expr, $s:ident => $body:expr) => {
        match $sound {
            VoiceSound::Static($s) => $body,
            VoiceSound::Stream($s) => $body,
        }
    };
}

impl VoiceSound {
    fn state(&self) -> PlaybackState {
        both!(self, s => s.state())
    }
    fn stop(&mut self, t: kira::Tween) {
        both!(self, s => s.stop(t))
    }
    fn set_volume(&mut self, db: Decibels, t: kira::Tween) {
        both!(self, s => s.set_volume(db, t))
    }
    fn set_panning(&mut self, p: Panning, t: kira::Tween) {
        both!(self, s => s.set_panning(p, t))
    }
    fn set_playback_rate(&mut self, r: PlaybackRate, t: kira::Tween) {
        both!(self, s => s.set_playback_rate(r, t))
    }
}

/// One playing placed or menu voice.
struct Voice {
    cue: &'static str,
    key: Option<LoopKey>,
    /// The voice's own sub-track (placed voices), dropped with it.
    _track: Option<TrackHandle>,
    filter: Option<kira::effect::filter::FilterHandle>,
    sound: VoiceSound,
    at: Where,
    db: f32,
    rate: f32,
    /// The distance law's gain last set, for the voice cap.
    gain: f32,
    pan: f32,
    cutoff: f32,
    asked: bool,
    stopping: bool,
}

/// A sound ready to play, decoded or streamed.
enum Playable {
    Static(kira::sound::static_sound::StaticSoundData),
    Stream(StreamingSoundData<FromFileError>),
}

impl Playable {
    /// A bed's file: streamed, or decoded whole under a capture.
    fn open(path: &std::path::Path, decode: bool) -> Result<Self, String> {
        if decode {
            kira::sound::static_sound::StaticSoundData::from_file(path)
                .map(Playable::Static)
                .map_err(|e| e.to_string())
        } else {
            StreamingSoundData::from_file(path)
                .map(Playable::Stream)
                .map_err(|e| e.to_string())
        }
    }

    fn duration_s(&self) -> f64 {
        match self {
            Playable::Static(d) => d.duration().as_secs_f64(),
            Playable::Stream(d) => d.duration().as_secs_f64(),
        }
    }

    /// Looped from `start` seconds, silent until eased up.
    fn bed(self, start: f64) -> Self {
        match self {
            Playable::Static(d) => Playable::Static(
                d.loop_region(..)
                    .start_position(start)
                    .volume(Decibels::SILENCE),
            ),
            Playable::Stream(d) => Playable::Stream(
                d.loop_region(..)
                    .start_position(start)
                    .volume(Decibels::SILENCE),
            ),
        }
    }

    fn play_on(self, track: &mut TrackHandle) -> Result<VoiceSound, String> {
        match self {
            Playable::Static(d) => track
                .play(d)
                .map(VoiceSound::Static)
                .map_err(|e| e.to_string()),
            Playable::Stream(d) => track
                .play(d)
                .map(VoiceSound::Stream)
                .map_err(|e| e.to_string()),
        }
    }
}

/// One playing bed.
struct BedVoice {
    sound: VoiceSound,
    /// Eased amplitude, linear.
    level: f32,
    rate: f32,
}

/// A small, seedable dice for takes, pitches and scatters: presentation
/// only, never anything saved.
struct Dice(u64);

impl Dice {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Everything kira, on the main thread.
pub struct Mixer {
    engine: Option<Engine>,
    bank: Arc<Mutex<Option<Bank>>>,
    loaded: Option<Bank>,
    voices: Vec<Voice>,
    beds: [Option<BedVoice>; Bed::COUNT],
    dice: Dice,
    last_take: HashMap<&'static str, usize>,
    late: BTreeSet<&'static str>,
    unknown: BTreeSet<&'static str>,
    log: bool,
    wav: Option<WavWriter>,
    volumes: Option<Volumes>,
    default_volumes: bool,
    focused: bool,
    world_state: (bool, bool, bool, i32, i32),
    since_meter: f32,
    started: u64,
    dropped: u64,
}

pub fn volume_db(percent: u32) -> f32 {
    if percent == 0 {
        -100.0
    } else {
        // A perceptual curve: 50 % is -12 dB.
        40.0 * (percent.min(100) as f32 / 100.0).log10()
    }
}

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        let launch = self.0.clone();
        let data = AudioData::load();
        let ready = SoundsReady::default();
        app.init_resource::<Sfx>()
            .init_resource::<Listener>()
            .init_resource::<Environment>()
            .insert_resource(data.clone())
            .insert_resource(ready.clone());
        hooks::register(app);
        let bank = Arc::new(Mutex::new(None));
        let mut mixer = Mixer {
            engine: None,
            bank: bank.clone(),
            loaded: None,
            voices: Vec::new(),
            beds: Default::default(),
            dice: Dice(if launch.capture {
                0x9E37_79B9_7F4A_7C15
            } else {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(1, |d| d.as_nanos() as u64)
                    | 1
            }),
            last_take: HashMap::new(),
            late: BTreeSet::new(),
            unknown: BTreeSet::new(),
            log: launch.log,
            wav: None,
            volumes: None,
            default_volumes: launch.default_volumes,
            focused: true,
            world_state: (false, false, false, 0, 0),
            since_meter: 0.0,
            started: 0,
            dropped: 0,
        };
        if launch.off {
            info!("AUDIO off: --no-audio");
            ready.0.store(true, Ordering::Release);
        } else {
            let backend = if launch.capture || launch.wav.is_some() && launch.capture {
                let step = 1.0 / crate::FIXED_HZ;
                Backend::Mock {
                    frames_per_step: (engine::MOCK_RATE as f64 * step).round() as usize,
                }
            } else {
                Backend::Device
            };
            match Engine::open(backend, launch.wav.is_some()) {
                Ok(engine) => {
                    info!(
                        "AUDIO on: {}",
                        if engine.is_mock() {
                            "the mock backend, stepped with the capture"
                        } else {
                            "the sound device"
                        }
                    );
                    if let Some(path) = &launch.wav {
                        let rate = if engine.is_mock() {
                            engine::MOCK_RATE
                        } else {
                            // The device's own rate is not known here; the
                            // files and most devices are 48 kHz.
                            48_000
                        };
                        match WavWriter::create(path, rate) {
                            Ok(w) => mixer.wav = Some(w),
                            Err(e) => warn!("AUDIO --audio-wav {}: {e}", path.display()),
                        }
                    }
                    mixer.engine = Some(engine);
                }
                Err(why) => {
                    warn!("AUDIO off: {why}; the game runs silent");
                }
            }
            // Decode the bank on its own thread from the first frame
            // (design decision 3): the loading screen waits on it.
            let cues = data.cues.clone();
            let flag = ready.0.clone();
            std::thread::Builder::new()
                .name("sound bank".into())
                .spawn(move || {
                    let decoded = engine::decode_bank(&engine::audio_dir().join("sfx"), &cues);
                    *bank.lock().unwrap_or_else(|e| e.into_inner()) = Some(decoded);
                    flag.store(true, Ordering::Release);
                })
                .expect("the sound bank thread starts");
        }
        app.insert_non_send_resource(mixer).add_systems(
            PostUpdate,
            (track_listener, mix_frame.after(track_listener)),
        );
    }
}

/// The listener from the active 3D camera.
fn track_listener(
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut listener: ResMut<Listener>,
) {
    let Some(frame) = frame else {
        return;
    };
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        listener.valid = false;
        return;
    };
    let eye = (transform.translation().as_dvec3() - frame.center).as_vec3();
    let rotation = transform.rotation();
    *listener = Listener {
        eye,
        inverse: rotation.inverse(),
        forward: rotation * Vec3::NEG_Z,
        valid: eye.is_finite() && eye != Vec3::ZERO,
    };
}

/// A placed sound's gain, pan and cutoff from where the ears are.
fn place(
    mix: &MixConfig,
    cue: &pbd_core::audio::Cue,
    listener: &Listener,
    at: Where,
) -> (f32, f32, f32, f32) {
    match at {
        Where::Menu => (1.0, 0.0, 20_000.0, 0.0),
        Where::At(point) => {
            let offset = point - listener.eye;
            let d = offset.length();
            let (p, behind) = pan(listener.inverse * offset);
            (
                distance_gain(d, cue.full_within_m, cue.audible_m),
                p,
                air_cutoff_hz(mix, d, behind),
                d,
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn mix_frame(
    mut mixer: NonSendMut<Mixer>,
    mut sfx: ResMut<Sfx>,
    listener: Res<Listener>,
    env: Res<Environment>,
    data: Res<AudioData>,
    settings: Option<Res<PlayerSettings>>,
    time: Res<Time>,
    warming: Option<Res<crate::loading::Warming>>,
    loading: Option<Res<crate::WorldLoading>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let requests = std::mem::take(&mut sfx.requests);
    let mixer = &mut *mixer;
    if mixer.loaded.is_none()
        && let Ok(mut slot) = mixer.bank.try_lock()
        && let Some(bank) = slot.take()
    {
        info!(
            "AUDIO bank {} files decoded in {:.2} s, {:.1} MB static",
            bank.files,
            bank.seconds,
            bank.bytes as f32 / 1_048_576.0
        );
        for why in &bank.failed {
            error!("AUDIO a file did not decode: {why}");
        }
        mixer.loaded = Some(bank);
    }
    let Some(engine) = mixer.engine.as_mut() else {
        if mixer.log {
            for r in &requests {
                if r.key.is_none() {
                    info!("AUDIO silent {}", r.cue);
                }
            }
        }
        return;
    };
    let dt = time.delta_secs();
    let mix = &data.mix;

    // The player's volumes, applied when they change.
    let volumes = if mixer.default_volumes {
        Volumes::default()
    } else {
        settings.map_or_else(Volumes::default, |s| s.volumes)
    };
    let focused = windows.iter().next().is_none_or(|w| w.focused) || engine.is_mock();
    let air_db = decibels(env.air.clamp(0.0, 1.0));
    let shelter_db = if env.sheltered { mix.shelter_db } else { 0.0 };
    // The world's state, rounded so a tiny change does not re-send.
    let state = (
        env.underwater,
        env.paused,
        env.ducked || !focused,
        (env.cave * 20.0).round() as i32,
        (air_db * 2.0).round() as i32 + (shelter_db * 10.0) as i32 * 1000,
    );
    if mixer.volumes != Some(volumes) || mixer.focused != focused || mixer.world_state != state {
        let fast = tween(0.15);
        engine.set_master(volume_db(volumes.master), fast);
        engine
            .effects
            .set_volume(Decibels(volume_db(volumes.effects) + air_db), fast);
        engine.ambience.set_volume(
            Decibels(volume_db(volumes.ambience) + shelter_db),
            tween(0.5),
        );
        engine.ambience_filter.set_cutoff(
            if env.sheltered {
                mix.shelter_cutoff_hz as f64
            } else {
                20_000.0
            },
            tween(0.5),
        );
        engine
            .menus
            .set_volume(Decibels(volume_db(volumes.menus)), fast);
        let music_db = if focused {
            volume_db(volumes.music)
        } else {
            -100.0
        };
        engine.music.set_volume(Decibels(music_db), tween(0.4));
        let (world_db, cutoff) = if !focused {
            (-100.0, 20_000.0)
        } else if env.underwater {
            (mix.underwater_db, mix.underwater_cutoff_hz)
        } else if env.ducked {
            (-6.0, 4_000.0)
        } else {
            (0.0, 20_000.0)
        };
        engine
            .world
            .set_volume(Decibels(world_db), tween(if focused { 0.5 } else { 0.3 }));
        engine.world_filter.set_cutoff(cutoff as f64, tween(0.15));
        let reverb = if env.underwater {
            mix.underwater_reverb
        } else {
            mix.outdoor_reverb + (mix.cave_reverb - mix.outdoor_reverb) * env.cave
        };
        let _ = engine.world.set_send(
            engine.reverb.id(),
            Decibels(decibels(reverb.max(1e-4))),
            tween(0.5),
        );
        if env.paused != mixer.world_state.1 || mixer.volumes.is_none() {
            if env.paused {
                engine.world.pause(tween(0.1));
            } else {
                engine.world.resume(tween(0.1));
            }
        }
        if mixer.log && mixer.volumes != Some(volumes) {
            info!(
                "AUDIO volumes master={} effects={} ambience={} music={} menus={}",
                volumes.master, volumes.effects, volumes.ambience, volumes.music, volumes.menus
            );
        }
        mixer.volumes = Some(volumes);
        mixer.focused = focused;
        mixer.world_state = state;
    }

    // The voices: ended ones go, loops not asked for fade, placed ones
    // follow the ears.
    mixer
        .voices
        .retain(|v| v.sound.state() != PlaybackState::Stopped);
    for voice in &mut mixer.voices {
        voice.asked = false;
    }
    let ready_now = warming.is_none() && loading.is_none();
    let Some(bank) = mixer.loaded.as_ref() else {
        if ready_now {
            for r in &requests {
                if mixer.late.insert(r.cue) {
                    warn!("LATE_SOUND {} (the bank is still decoding)", r.cue);
                }
            }
        }
        return;
    };
    for request in requests {
        let Some(cue) = data.cues.cues.get(request.cue) else {
            if mixer.unknown.insert(request.cue) {
                error!("AUDIO no cue {} in cues.ron", request.cue);
            }
            continue;
        };
        if let Some(key) = request.key
            && let Some(voice) = mixer
                .voices
                .iter_mut()
                .find(|v| v.key == Some(key) && !v.stopping)
        {
            voice.asked = true;
            voice.at = request.at;
            voice.db = cue.volume_db + request.db;
            voice.rate = request.rate;
            continue;
        }
        let takes: &[kira::sound::static_sound::StaticSoundData] = if cue.stream {
            &[]
        } else {
            match bank.sounds.get(request.cue).filter(|t| !t.is_empty()) {
                Some(takes) => takes,
                None => {
                    if mixer.late.insert(request.cue) {
                        warn!("LATE_SOUND {}", request.cue);
                    }
                    continue;
                }
            }
        };
        let take_count = if cue.stream {
            cue.takes.max(1) as usize
        } else {
            takes.len()
        };
        if !listener.valid && matches!(request.at, Where::At(_)) {
            continue;
        }
        let (gain, p, cutoff, d) = place(mix, cue, &listener, request.at);
        let db = cue.volume_db + request.db;
        if gain * amplitude(db) < 1e-3 && !cue.looped {
            continue;
        }
        // The cue's cap, then the mixer's.
        let same: Vec<(usize, f32)> = mixer
            .voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.cue == request.cue && !v.stopping)
            .map(|(i, v)| (i, v.gain * amplitude(v.db)))
            .collect();
        let loud = gain * amplitude(db);
        let gains: Vec<f32> = same.iter().map(|(_, g)| *g).collect();
        let mut replace = None;
        match admit(&gains, cue.cap as usize, loud) {
            Admit::Play => {}
            Admit::Replace(i) => replace = Some(same[i].0),
            Admit::Drop => {
                mixer.dropped += 1;
                if mixer.log {
                    info!("AUDIO drop {} (its cap {})", request.cue, cue.cap);
                }
                continue;
            }
        }
        if replace.is_none() && cue.bus == Bus::Effects {
            let placed: Vec<(usize, f32)> = mixer
                .voices
                .iter()
                .enumerate()
                .filter(|(_, v)| v.filter.is_some() && !v.stopping)
                .map(|(i, v)| (i, v.gain * amplitude(v.db)))
                .collect();
            let gains: Vec<f32> = placed.iter().map(|(_, g)| *g).collect();
            match admit(&gains, mix.max_voices, loud) {
                Admit::Play => {}
                Admit::Replace(i) => replace = Some(placed[i].0),
                Admit::Drop => {
                    mixer.dropped += 1;
                    if mixer.log {
                        info!("AUDIO drop {} (the mixer is full)", request.cue);
                    }
                    continue;
                }
            }
        }
        if let Some(i) = replace {
            let voice = &mut mixer.voices[i];
            voice.sound.stop(tween(0.05));
            voice.stopping = true;
        }
        // A take, never the same twice running.
        let mut take = (mixer.dice.next() % take_count as u64) as usize;
        if take_count > 1 && mixer.last_take.get(request.cue) == Some(&take) {
            take = (take + 1) % take_count;
        }
        mixer.last_take.insert(request.cue, take);
        let rate = request.rate * (cue.pitch.0 + (cue.pitch.1 - cue.pitch.0) * mixer.dice.unit());
        let delay = match request.at {
            Where::At(_) => (delay_s(mix, d) - request.ago_s).max(0.0),
            Where::Menu => 0.0,
        };
        let start = if delay > 0.0 {
            StartTime::Delayed(Duration::from_secs_f32(delay))
        } else {
            StartTime::Immediate
        };
        let loop_from = mixer.dice.unit() as f64;
        // A capture decodes what is otherwise streamed, so the mock mixer
        // never waits on a stream's thread and its WAV is the same each run.
        let sound = if cue.stream && engine.is_mock() {
            let file = &cue.files(request.cue)[take];
            match kira::sound::static_sound::StaticSoundData::from_file(
                engine::audio_dir().join("sfx").join(file),
            ) {
                Ok(data) => {
                    let mut data = data
                        .volume(Decibels(db + decibels(gain)))
                        .panning(Panning(p))
                        .playback_rate(PlaybackRate(rate as f64))
                        .start_time(start);
                    if cue.looped {
                        let length = data.duration().as_secs_f64();
                        data = data
                            .loop_region(..)
                            .start_position(loop_from * length)
                            .fade_in_tween(Some(tween(mix.loop_fade_s)));
                    }
                    Playable::Static(data)
                }
                Err(e) => {
                    if mixer.unknown.insert(request.cue) {
                        error!("AUDIO {file}: {e}");
                    }
                    continue;
                }
            }
        } else if cue.stream {
            let file = &cue.files(request.cue)[take];
            match StreamingSoundData::from_file(engine::audio_dir().join("sfx").join(file)) {
                Ok(data) => {
                    let mut data = data
                        .volume(Decibels(db + decibels(gain)))
                        .panning(Panning(p))
                        .playback_rate(PlaybackRate(rate as f64))
                        .start_time(start);
                    if cue.looped {
                        let length = data.duration().as_secs_f64();
                        data = data
                            .loop_region(..)
                            .start_position(loop_from * length)
                            .fade_in_tween(Some(tween(mix.loop_fade_s)));
                    }
                    Playable::Stream(data)
                }
                Err(e) => {
                    if mixer.unknown.insert(request.cue) {
                        error!("AUDIO {file}: {e}");
                    }
                    continue;
                }
            }
        } else {
            let mut data = takes[take]
                .volume(Decibels(db + decibels(gain)))
                .panning(Panning(p))
                .playback_rate(PlaybackRate(rate as f64))
                .start_time(start);
            if cue.looped {
                let length = data.duration().as_secs_f64();
                data = data
                    .loop_region(..)
                    .start_position(loop_from * length)
                    .fade_in_tween(Some(tween(mix.loop_fade_s)));
            }
            Playable::Static(data)
        };
        let played = match cue.bus {
            Bus::Menus => sound.play_on(&mut engine.menus).map(|s| (None, None, s)),
            Bus::Effects => {
                let mut builder = TrackBuilder::new().persist_until_sounds_finish(true);
                let filter = builder
                    .add_effect(kira::effect::filter::FilterBuilder::new().cutoff(cutoff as f64));
                match engine.effects.add_sub_track(builder) {
                    Ok(mut track) => sound
                        .play_on(&mut track)
                        .map(|s| (Some(track), Some(filter), s)),
                    Err(e) => {
                        warn!("AUDIO {}: {e}", request.cue);
                        continue;
                    }
                }
            }
        };
        match played {
            Ok((track, filter, sound)) => {
                mixer.started += 1;
                if mixer.log {
                    match request.at {
                        Where::Menu => info!(
                            "AUDIO start {} take {} menu {:+.1} dB",
                            request.cue,
                            take + 1,
                            db
                        ),
                        Where::At(_) => info!(
                            "AUDIO start {} take {} at {:.1} m gain {:+.1} dB cutoff {:.1} kHz pan {:+.2} delay {:.0} ms",
                            request.cue,
                            take + 1,
                            d,
                            db + decibels(gain),
                            cutoff / 1000.0,
                            p,
                            delay * 1000.0
                        ),
                    }
                }
                mixer.voices.push(Voice {
                    cue: request.cue,
                    key: request.key,
                    _track: track,
                    filter,
                    sound,
                    at: request.at,
                    db,
                    rate: request.rate,
                    gain,
                    pan: p,
                    cutoff,
                    asked: true,
                    stopping: false,
                });
            }
            Err(e) => warn!("AUDIO {}: {e}", request.cue),
        }
    }
    let step = tween(0.05);
    for voice in &mut mixer.voices {
        if voice.stopping {
            continue;
        }
        if voice.key.is_some() && !voice.asked {
            voice.sound.stop(tween(mix.loop_fade_s));
            voice.stopping = true;
            if mixer.log {
                info!("AUDIO stop {}", voice.cue);
            }
            continue;
        }
        let Some(cue) = data.cues.cues.get(voice.cue) else {
            continue;
        };
        let (gain, p, cutoff, _) = place(mix, cue, &listener, voice.at);
        if (decibels(gain) - decibels(voice.gain)).abs() > 0.25 || voice.key.is_some() {
            voice
                .sound
                .set_volume(Decibels(voice.db + decibels(gain)), step);
            voice.gain = gain;
        }
        if (p - voice.pan).abs() > 0.02 {
            voice.sound.set_panning(Panning(p), step);
            voice.pan = p;
        }
        if let Some(filter) = &mut voice.filter
            && (cutoff / voice.cutoff).ln().abs() > 0.05
        {
            filter.set_cutoff(cutoff as f64, step);
            voice.cutoff = cutoff;
        }
        if voice.key.is_some() {
            voice
                .sound
                .set_playback_rate(PlaybackRate(voice.rate as f64), step);
        }
    }

    // The beds, eased toward the last read's targets.
    if let Some(targets) = env.beds {
        let ease = 1.0 - (-dt / mix.bed_ease_s.max(1e-3)).exp();
        for bed in Bed::ALL {
            let level_db = data
                .ambience
                .beds
                .get(bed.name())
                .map_or(-6.0, |b| b.volume_db);
            let target = targets.weight(bed).sqrt() * amplitude(level_db);
            let rate = if matches!(bed, Bed::WindLow | Bed::WindHigh) {
                targets.wind_rate
            } else {
                1.0
            };
            let slot = &mut mixer.beds[bed.index()];
            match slot {
                None if target > 0.01 => {
                    let path = engine::audio_dir()
                        .join("ambience")
                        .join(format!("{}.ogg", bed.name()));
                    match Playable::open(&path, engine.is_mock()) {
                        Ok(data) => {
                            let length = data.duration_s();
                            let start = mixer.dice.unit() as f64 * length;
                            match data.bed(start).play_on(&mut engine.ambience) {
                                Ok(sound) => {
                                    if mixer.log {
                                        info!("AUDIO bed start {} at {:.1} s", bed.name(), start);
                                    }
                                    *slot = Some(BedVoice {
                                        sound,
                                        level: 0.0,
                                        rate: 1.0,
                                    });
                                }
                                Err(e) => warn!("AUDIO bed {}: {e}", bed.name()),
                            }
                        }
                        Err(e) => warn!("AUDIO bed {}: {e}", path.display()),
                    }
                }
                Some(voice) => {
                    voice.level += (target - voice.level) * ease;
                    if voice.level < 0.01 && target < 0.01 {
                        voice.sound.stop(tween(0.3));
                        if mixer.log {
                            info!("AUDIO bed stop {}", bed.name());
                        }
                        *slot = None;
                    } else {
                        voice
                            .sound
                            .set_volume(Decibels(decibels(voice.level)), tween(0.25));
                        if (voice.rate - rate).abs() > 0.005 {
                            voice
                                .sound
                                .set_playback_rate(PlaybackRate(rate as f64), tween(0.5));
                            voice.rate = rate;
                        }
                    }
                }
                None => {}
            }
        }
    }

    // The instruments.
    if mixer.log {
        mixer.since_meter += dt;
        if mixer.since_meter >= 1.0 {
            mixer.since_meter = 0.0;
            let meters: Vec<String> = engine
                .meters
                .iter()
                .map(|(name, m)| {
                    let (peak, rms) = m.take();
                    format!("{name} {:.0}/{:.0}", decibels(peak), decibels(rms))
                })
                .collect();
            let beds: Vec<String> = Bed::ALL
                .iter()
                .filter_map(|b| {
                    mixer.beds[b.index()].as_mut().map(|v| {
                        if let VoiceSound::Stream(stream) = &mut v.sound
                            && let Some(e) = stream.pop_error()
                        {
                            warn!("AUDIO bed {} stream error: {e}", b.name());
                        }
                        let state = match v.sound.state() {
                            PlaybackState::Playing => "",
                            PlaybackState::Stopped => " STOPPED",
                            _ => " (paused)",
                        };
                        format!("{}={:.0}{state}", &b.name()[4..], decibels(v.level))
                    })
                })
                .collect();
            info!(
                "AUDIO buses peak/rms dB: {} | voices {} started {} dropped {} | beds {}",
                meters.join(", "),
                mixer.voices.len(),
                mixer.started,
                mixer.dropped,
                beds.join(" ")
            );
        }
        if let Some(cpu) = engine.cpu_usage() {
            debug!("AUDIO cpu {:.1}%", cpu * 100.0);
        }
    }
    if let Some(why) = engine.device_error() {
        warn!("AUDIO the sound device reported: {why}; kira restarts the stream");
    }
    // The mock backend mixes one frame's samples a frame from `READY`, when
    // a capture's frames start counting, so the WAV is in step with them.
    if ready_now {
        engine.advance();
    }
    if let (Some(tap), Some(wav)) = (engine.tap.as_ref(), mixer.wav.as_mut()) {
        let samples = std::mem::take(&mut *tap.lock().unwrap_or_else(|e| e.into_inner()));
        if let Err(e) = wav.append(&samples) {
            warn!("AUDIO --audio-wav: {e}");
            mixer.wav = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Task 4.3: only `emitter - eye` reaches the engine, so moving the whole
    /// scene (a floating-origin rebase) moves no sound.
    #[test]
    fn an_origin_rebase_moves_no_sound() {
        let mix = MixConfig::default();
        let cue = pbd_core::audio::Cue {
            full_within_m: 2.0,
            audible_m: 30.0,
            ..Default::default()
        };
        let turn = Quat::from_rotation_y(0.7);
        let at = |shift: Vec3| {
            let listener = Listener {
                eye: Vec3::new(10.0, 4800.0, -3.0) + shift,
                inverse: turn.inverse(),
                forward: turn * Vec3::NEG_Z,
                valid: true,
            };
            place(
                &mix,
                &cue,
                &listener,
                Where::At(Vec3::new(30.0, 4801.0, 12.0) + shift),
            )
        };
        let (g0, p0, c0, d0) = at(Vec3::ZERO);
        let (g1, p1, c1, d1) = at(Vec3::new(4096.0, -2048.0, 1024.0));
        assert!(
            (g0 - g1).abs() < 1e-3 && (p0 - p1).abs() < 1e-3,
            "{g0} {g1} {p0} {p1}"
        );
        assert!((c0 - c1).abs() / c0 < 1e-3 && (d0 - d1).abs() < 1e-2);
        assert!(
            g0 > 0.0 && p0.abs() > 0.1,
            "a placed, panned sound: {g0} {p0}"
        );
    }

    #[test]
    fn a_volume_of_nought_is_silent_and_full_is_unity() {
        assert_eq!(volume_db(100), 0.0);
        assert!(volume_db(0) <= -100.0);
        assert!((volume_db(50) + 12.04).abs() < 0.01);
    }
}
