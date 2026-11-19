//! MEASUREMENT (`memory-leak`), behind the `alloc-trace` cargo feature: the
//! process's allocator counts every allocation of `PBD_ALLOC_TRACE_MIN` bytes
//! or more (256 KiB by default) by its call stack, walked off the frame
//! pointers, and writes the table to `PBD_ALLOC_TRACE=<path>` once a minute.
//! `tools/alloc_trace.py` turns the addresses into names with `atos`. It is
//! how memory that the allocator keeps after it is freed is traced to the
//! code that keeps asking for it. Without the feature none of this is built;
//! with it and no `PBD_ALLOC_TRACE`, every allocation passes straight through.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    io::Write as _,
    sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

const SLOTS: usize = 8192;
const DEPTH: usize = 24;

struct Slot {
    key: AtomicU64,
    count: AtomicU64,
    bytes: AtomicU64,
    ips: [AtomicUsize; DEPTH],
}

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY: Slot = Slot {
    key: AtomicU64::new(0),
    count: AtomicU64::new(0),
    bytes: AtomicU64::new(0),
    ips: [const { AtomicUsize::new(0) }; DEPTH],
};

static TABLE: [Slot; SLOTS] = [EMPTY; SLOTS];
static ON: AtomicBool = AtomicBool::new(false);
static MIN: AtomicUsize = AtomicUsize::new(256 * 1024);
static DROPPED: AtomicU64 = AtomicU64::new(0);

pub struct Tracing;

#[global_allocator]
static GLOBAL: Tracing = Tracing;

unsafe extern "C" {
    fn _dyld_get_image_vmaddr_slide(image_index: u32) -> isize;
}

/// The caller's return addresses, walked off the frame-pointer chain.
#[inline(always)]
fn walk(out: &mut [usize; DEPTH]) -> usize {
    let mut fp: usize;
    #[cfg(target_arch = "aarch64")]
    unsafe {
        std::arch::asm!("mov {}, x29", out(reg) fp);
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        fp = 0;
    }
    let start = fp;
    let mut n = 0;
    while n < DEPTH && fp != 0 && fp % 8 == 0 && fp >= start && fp - start < (8 << 20) {
        // SAFETY: a frame record on this thread's stack: [fp] the caller's
        // frame pointer, [fp + 8] the return address. The bounds above keep
        // the walk inside the stack the walk started on.
        let (next, lr) = unsafe { (*(fp as *const usize), *((fp + 8) as *const usize)) };
        if lr == 0 {
            break;
        }
        out[n] = lr;
        n += 1;
        if next <= fp {
            break;
        }
        fp = next;
    }
    n
}

fn record(size: usize) {
    let mut ips = [0usize; DEPTH];
    let n = walk(&mut ips);
    let mut key: u64 = 0xcbf2_9ce4_8422_2325;
    for ip in &ips[..n] {
        key = (key ^ *ip as u64).wrapping_mul(0x0100_0000_01b3);
    }
    key |= 1;
    let mut i = (key as usize) % SLOTS;
    for _ in 0..SLOTS {
        let slot = &TABLE[i];
        match slot
            .key
            .compare_exchange(0, key, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => {
                for (k, ip) in ips.iter().enumerate() {
                    slot.ips[k].store(*ip, Ordering::Relaxed);
                }
                slot.count.fetch_add(1, Ordering::Relaxed);
                slot.bytes.fetch_add(size as u64, Ordering::Relaxed);
                return;
            }
            Err(found) if found == key => {
                slot.count.fetch_add(1, Ordering::Relaxed);
                slot.bytes.fetch_add(size as u64, Ordering::Relaxed);
                return;
            }
            Err(_) => i = (i + 1) % SLOTS,
        }
    }
    DROPPED.fetch_add(1, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for Tracing {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ON.load(Ordering::Relaxed) && layout.size() >= MIN.load(Ordering::Relaxed) {
            record(layout.size());
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if ON.load(Ordering::Relaxed) && layout.size() >= MIN.load(Ordering::Relaxed) {
            record(layout.size());
        }
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if ON.load(Ordering::Relaxed) && new_size >= MIN.load(Ordering::Relaxed) {
            record(new_size);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// Starts the count and the minute-by-minute writer, if `PBD_ALLOC_TRACE`
/// names a file.
pub fn start() {
    let Some(path) = std::env::var_os("PBD_ALLOC_TRACE") else {
        return;
    };
    if let Some(min) = std::env::var("PBD_ALLOC_TRACE_MIN")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        MIN.store(min, Ordering::Relaxed);
    }
    ON.store(true, Ordering::Relaxed);
    let started = std::time::Instant::now();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
            let Ok(mut out) = std::fs::File::create(&path) else {
                continue;
            };
            // SAFETY: a dyld query with no preconditions; image 0 is the
            // executable.
            let slide = unsafe { _dyld_get_image_vmaddr_slide(0) };
            let _ = writeln!(
                out,
                "# since_s {:.0} slide {:#x} dropped {}",
                started.elapsed().as_secs_f64(),
                slide,
                DROPPED.load(Ordering::Relaxed)
            );
            for slot in &TABLE {
                if slot.key.load(Ordering::Acquire) == 0 {
                    continue;
                }
                let ips: Vec<String> = slot
                    .ips
                    .iter()
                    .map(|ip| ip.load(Ordering::Relaxed))
                    .take_while(|ip| *ip != 0)
                    .map(|ip| format!("{ip:#x}"))
                    .collect();
                let _ = writeln!(
                    out,
                    "{} {} {}",
                    slot.count.load(Ordering::Relaxed),
                    slot.bytes.load(Ordering::Relaxed),
                    ips.join(" ")
                );
            }
        }
    });
}
