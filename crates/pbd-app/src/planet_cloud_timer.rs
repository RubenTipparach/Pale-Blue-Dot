//! The clouds' GPU time where Bevy's pass spans cannot measure it
//! (`cloud-puffs`, a measurement instrument). Bevy writes a span's timestamps
//! INSIDE its pass, which Apple GPUs do not support (Metal samples counters at
//! stage boundaries only), so on a Mac every span read nought. Here each cloud
//! pass is given `timestamp_writes` at its own beginning and end, the one form
//! Metal has (the counterpart of `MTLCommandBuffer.GPUStartTime`/`GPUEndTime`
//! per pass), resolved and read back a few frames late.
//!
//! Only `--frame-log` runs read it ([`cloud_gpu_ms`]); without the timestamp
//! feature it stays off and reads `None`.

use bevy::prelude::*;
use bevy::render::{
    render_resource::{
        Buffer, BufferDescriptor, BufferUsages, CommandEncoder, MapMode, WgpuFeatures,
    },
    renderer::{RenderDevice, RenderQueue},
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use wgpu::{QuerySet, QuerySetDescriptor, QueryType, RenderPassTimestampWrites};

/// The cloud passes timed: the march, the puffs or the resolve, the composite.
pub const PASSES: u32 = 3;
const QUERIES: u32 = PASSES * 2;
const BYTES: u64 = QUERIES as u64 * 8;
/// A pass's pair is resolved at its own offset, which wgpu wants aligned to
/// `QUERY_RESOLVE_BUFFER_ALIGNMENT` (256 bytes).
const RESOLVE_STRIDE: u64 = 256;
/// Read-back buffers in flight: a frame's stamps are read a few frames on.
const RING: usize = 3;

/// The newest measured cloud GPU time, ms, as f64 bits; NaN before the first.
static CLOUD_GPU_MS: AtomicU64 = AtomicU64::new(0x7FF8_0000_0000_0000);

/// The clouds' GPU time, ms, from the newest frame read back, when the timer
/// runs.
pub fn cloud_gpu_ms() -> Option<f64> {
    let ms = f64::from_bits(CLOUD_GPU_MS.load(Ordering::Relaxed));
    ms.is_finite().then_some(ms)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Slot {
    Free,
    /// Copied into this frame; to be mapped once submitted.
    Written,
    Mapping,
}

struct Ring {
    slots: [Slot; RING],
    /// The slot the node writes this frame, if one was free.
    writing: Option<usize>,
    /// Which of this frame's passes ran (a pass that did not leaves its pair
    /// unwritten, and its stamps mean nothing).
    ran: [bool; PASSES as usize],
}

/// The query set, its resolve buffer and the read-back ring.
#[derive(Resource)]
pub(super) struct CloudTimer {
    set: QuerySet,
    resolve: Buffer,
    readback: [Buffer; RING],
    ready: [Arc<std::sync::atomic::AtomicBool>; RING],
    /// Which passes each slot's frame ran.
    ran: Mutex<[[bool; PASSES as usize]; RING]>,
    ring: Mutex<Ring>,
    period_ns: f32,
}

impl CloudTimer {
    /// The pass's timestamp writes, when this frame is being timed.
    pub fn writes(&self, pass: u32) -> Option<RenderPassTimestampWrites<'_>> {
        let mut ring = self.ring.lock().ok()?;
        ring.writing?;
        ring.ran[pass as usize] = true;
        Some(RenderPassTimestampWrites {
            query_set: &self.set,
            beginning_of_pass_write_index: Some(pass * 2),
            end_of_pass_write_index: Some(pass * 2 + 1),
        })
    }

    /// Takes a free slot for this frame's stamps, before the passes run.
    pub fn begin_frame(&self) {
        if let Ok(mut ring) = self.ring.lock() {
            ring.writing = ring.slots.iter().position(|s| *s == Slot::Free);
            ring.ran = [false; PASSES as usize];
        }
    }

    /// Resolves this frame's stamps into its slot, after the passes.
    pub fn end_frame(&self, encoder: &mut CommandEncoder) {
        let Ok(mut ring) = self.ring.lock() else {
            return;
        };
        let Some(slot) = ring.writing.take() else {
            return;
        };
        if !ring.ran.iter().any(|r| *r) {
            return;
        }
        // Only the pairs written this frame: the rest are left out.
        for pass in 0..PASSES {
            if ring.ran[pass as usize] {
                let at = pass as u64 * RESOLVE_STRIDE;
                encoder.resolve_query_set(&self.set, pass * 2..pass * 2 + 2, &self.resolve, at);
                encoder.copy_buffer_to_buffer(
                    &self.resolve,
                    at,
                    &self.readback[slot],
                    pass as u64 * 16,
                    16,
                );
            }
        }
        if let Ok(mut ran) = self.ran.lock() {
            ran[slot] = ring.ran;
        }
        ring.slots[slot] = Slot::Written;
    }
}

pub(super) fn init(mut commands: Commands, device: Res<RenderDevice>, queue: Res<RenderQueue>) {
    if !device.features().contains(WgpuFeatures::TIMESTAMP_QUERY) {
        return;
    }
    let set = device.wgpu_device().create_query_set(&QuerySetDescriptor {
        label: Some("cloud pass timestamps"),
        ty: QueryType::Timestamp,
        count: QUERIES,
    });
    let resolve = device.create_buffer(&BufferDescriptor {
        label: Some("cloud pass timestamps resolved"),
        size: PASSES as u64 * RESOLVE_STRIDE,
        usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = std::array::from_fn(|_| {
        device.create_buffer(&BufferDescriptor {
            label: Some("cloud pass timestamps read back"),
            size: BYTES,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    });
    commands.insert_resource(CloudTimer {
        set,
        resolve,
        readback,
        ready: std::array::from_fn(|_| Arc::default()),
        ran: Mutex::new([[false; PASSES as usize]; RING]),
        ring: Mutex::new(Ring {
            slots: [Slot::Free; RING],
            writing: None,
            ran: [false; PASSES as usize],
        }),
        period_ns: queue.get_timestamp_period(),
    });
}

/// After the frame is submitted: maps the slots written this frame, and reads
/// and frees the ones whose mapping has come back.
pub(super) fn read_back(timer: Option<Res<CloudTimer>>) {
    let Some(timer) = timer else {
        return;
    };
    let Ok(mut ring) = timer.ring.lock() else {
        return;
    };
    for slot in 0..RING {
        match ring.slots[slot] {
            Slot::Written => {
                let ready = timer.ready[slot].clone();
                ready.store(false, Ordering::Release);
                timer.readback[slot]
                    .slice(..)
                    .map_async(MapMode::Read, move |result| {
                        if result.is_ok() {
                            ready.store(true, Ordering::Release);
                        }
                    });
                ring.slots[slot] = Slot::Mapping;
            }
            Slot::Mapping if timer.ready[slot].load(Ordering::Acquire) => {
                let ran = timer.ran.lock().map(|r| r[slot]).unwrap_or_default();
                let stamps: Vec<u64> = {
                    let view = timer.readback[slot].slice(..).get_mapped_range();
                    bytemuck::cast_slice::<u8, u64>(&view).to_vec()
                };
                timer.readback[slot].unmap();
                // From the first cloud pass's start to the last one's end. An
                // Apple GPU overlaps consecutive passes (one's vertex work
                // with the last one's fragments), so a pass's own pair
                // brackets its neighbours' work as well and the pairs do not
                // add up; the span over all of them does not overlap itself.
                let ticks_of = |p: usize| (stamps[p * 2], stamps[p * 2 + 1]);
                let begin = (0..PASSES as usize)
                    .filter(|&p| ran[p])
                    .map(|p| ticks_of(p).0)
                    .min();
                let end = (0..PASSES as usize)
                    .filter(|&p| ran[p])
                    .map(|p| ticks_of(p).1)
                    .max();
                if let (Some(begin), Some(end)) = (begin, end)
                    && end > begin
                {
                    let ms = (end - begin) as f64 * f64::from(timer.period_ns) / 1.0e6;
                    CLOUD_GPU_MS.store(ms.to_bits(), Ordering::Relaxed);
                }
                ring.slots[slot] = Slot::Free;
            }
            _ => {}
        }
    }
}
