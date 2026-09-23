//! One thread owns the disk, and the frame never waits for it.
//!
//! `CLAUDE.md` demands that "every accepted world mutation enters the durable
//! transaction path immediately" and that commitment is acknowledged "only
//! after the storage backend succeeds"; the owner asks for saving to be fully
//! async. Those are the same requirement once the WRITE is separated from the
//! ACKNOWLEDGEMENT, and the rule's own wording says so: it forbids
//! acknowledging on the enqueue, not writing off the frame.
//!
//! So the game sends a record and returns. The thread writes it, fsyncs, and
//! publishes a high-water sequence mark; anything at or below the mark is on
//! disk. Nothing else in the process opens these files.
//!
//! Draining is what async BUYS rather than merely what it costs: holding a
//! button through a wall of dirt used to be an fsync per cell, and is one
//! fsync per batch now.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// How long a drain waits for the disk before giving up and saying so.
/// Tenebris's `flush_blocking` figure, for its reason: a wedged filesystem
/// must not be able to hang a shutdown.
const DRAIN_LIMIT: Duration = Duration::from_secs(2);

/// What the game hands the thread.
enum Job {
    /// Append this line to the slot's log.
    Append { seq: u64, line: String },
    /// Replace this file's whole contents, via a temporary and a rename, so a
    /// torn write cannot leave half a snapshot where a whole one was.
    Replace {
        seq: u64,
        path: PathBuf,
        body: Vec<u8>,
    },
    /// Write to this slot from now on.
    Slot(PathBuf),
    /// Everything before this is on disk; say so down the channel.
    Barrier(Sender<()>),
}

/// The queue, the mark, and the thread behind them.
pub struct SaveWriter {
    jobs: Option<Sender<Job>>,
    committed: Arc<AtomicU64>,
    failure: Arc<Mutex<Option<String>>>,
    queued: u64,
    thread: Option<JoinHandle<()>>,
}

impl SaveWriter {
    /// Start the thread on a slot directory.
    pub fn new(slot: impl AsRef<Path>) -> Self {
        let (jobs, inbox) = channel();
        let committed = Arc::new(AtomicU64::new(0));
        let failure = Arc::new(Mutex::new(None));
        let thread = {
            let committed = Arc::clone(&committed);
            let failure = Arc::clone(&failure);
            let slot = slot.as_ref().to_path_buf();
            std::thread::Builder::new()
                .name("pbd-saves".into())
                .spawn(move || run(inbox, slot, committed, failure))
                .expect("the save thread")
        };
        Self {
            jobs: Some(jobs),
            committed,
            failure,
            queued: 0,
            thread: Some(thread),
        }
    }

    /// A writer with no thread, for a run that must not touch the disk: the
    /// capture harness and the tests. Everything sent to it is dropped, and
    /// `pending` stays at nought, because nothing was ever queued.
    pub fn none() -> Self {
        Self {
            jobs: None,
            committed: Arc::new(AtomicU64::new(0)),
            failure: Arc::new(Mutex::new(None)),
            queued: 0,
            thread: None,
        }
    }

    fn send(&mut self, make: impl FnOnce(u64) -> Job) -> u64 {
        let Some(jobs) = self.jobs.as_ref() else {
            return 0;
        };
        self.queued += 1;
        let seq = self.queued;
        if jobs.send(make(seq)).is_err() {
            // The thread is gone, which is a failure the screen has to show
            // rather than a line that quietly stopped being written.
            *self.failure.lock().unwrap() = Some("the save thread stopped".into());
        }
        seq
    }

    /// Queue a line for the slot's log. Returns its sequence; it is SAVED when
    /// `committed()` reaches it, and not before.
    pub fn append(&mut self, line: String) -> u64 {
        self.send(|seq| Job::Append { seq, line })
    }

    /// Queue a whole-file replacement.
    pub fn replace(&mut self, path: PathBuf, body: impl Into<Vec<u8>>) -> u64 {
        let body = body.into();
        self.send(|seq| Job::Replace { seq, path, body })
    }

    /// Write to a different slot from here on. Everything queued before it
    /// still lands in the old one, which is what makes switching safe: the
    /// jobs are one ordered queue.
    pub fn use_slot(&mut self, slot: PathBuf) {
        if let Some(jobs) = self.jobs.as_ref() {
            let _ = jobs.send(Job::Slot(slot));
        }
    }

    /// The highest sequence the disk has taken.
    pub fn committed(&self) -> u64 {
        self.committed.load(Ordering::Acquire)
    }

    /// How many records are queued and not yet down.
    pub fn pending(&self) -> u64 {
        self.queued.saturating_sub(self.committed())
    }

    /// What went wrong, if anything has. A save that is silently not happening
    /// is the worst state this can be in, so the screen reads this.
    pub fn failure(&self) -> Option<String> {
        self.failure.lock().unwrap().clone()
    }

    /// Wait until everything queued so far is on disk.
    ///
    /// The one place blocking is right is the place the player is already
    /// waiting. Losing the last two digs to a quit would be the feature
    /// failing at its most visible moment.
    ///
    /// **Capped, which is Tenebris's own lesson in its `flush_blocking`**: a
    /// wedged filesystem must not be able to hang a shutdown. Two seconds and
    /// a line saying the last write may be stale beats a window that will not
    /// close, and the line is what turns a hang into a report.
    pub fn drain(&self) {
        let Some(jobs) = self.jobs.as_ref() else {
            return;
        };
        let (done, wait) = channel();
        if jobs.send(Job::Barrier(done)).is_ok() && wait.recv_timeout(DRAIN_LIMIT).is_err() {
            *self.failure.lock().unwrap() =
                Some("the save timed out; the last write may be stale".into());
        }
    }
}

impl Drop for SaveWriter {
    fn drop(&mut self) {
        self.drain();
        // Dropping the sender is what ends the loop; the join is what makes
        // "the process exited" mean "the file is closed".
        self.jobs = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Open the slot's log for appending, making the directory if it is new.
fn log_file(slot: &Path) -> std::io::Result<File> {
    std::fs::create_dir_all(slot)?;
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(slot.join(super::LOG))
}

/// Replace a file whole: write a temporary beside it, sync it, and rename.
/// A rename within a directory is atomic, so a reader sees the old file or the
/// new one and never half of either.
fn replace_file(path: &Path, body: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    {
        let mut file = File::create(&temporary)?;
        file.write_all(body)?;
        file.sync_data()?;
    }
    std::fs::rename(&temporary, path)
}

fn run(
    inbox: Receiver<Job>,
    mut slot: PathBuf,
    committed: Arc<AtomicU64>,
    failure: Arc<Mutex<Option<String>>>,
) {
    let note = |error: std::io::Error, what: &str| {
        *failure.lock().unwrap() = Some(format!("{what}: {error}"));
    };
    while let Ok(first) = inbox.recv() {
        // Everything already waiting goes down with the first, under ONE
        // fsync. This is the whole of what moving the write off the frame
        // bought, and it is why the mark is a high-water number: the jobs are
        // written in the order they were sent.
        let batch: Vec<Job> = std::iter::once(first).chain(inbox.try_iter()).collect();
        let mut appended: Option<File> = None;
        let mut mark = 0;
        let mut barriers = Vec::new();
        for job in batch {
            match job {
                Job::Slot(next) => {
                    // Close the old log before the new one opens, so the two
                    // are never both live.
                    if let Some(file) = appended.take() {
                        let _ = file.sync_data();
                    }
                    slot = next;
                }
                Job::Append { seq, line } => {
                    if appended.is_none() {
                        match log_file(&slot) {
                            Ok(file) => appended = Some(file),
                            Err(error) => {
                                note(error, "opening the log");
                                continue;
                            }
                        }
                    }
                    let Some(file) = appended.as_mut() else {
                        continue;
                    };
                    match file.write_all(line.as_bytes()) {
                        Ok(()) => mark = mark.max(seq),
                        Err(error) => note(error, "writing the log"),
                    }
                }
                Job::Replace { seq, path, body } => match replace_file(&path, &body) {
                    Ok(()) => mark = mark.max(seq),
                    Err(error) => note(error, "writing the snapshot"),
                },
                Job::Barrier(done) => barriers.push(done),
            }
        }
        if let Some(file) = appended.as_mut()
            && let Err(error) = file.sync_data()
        {
            note(error, "syncing the log");
            // The write is not down, so the mark must not move: a sequence
            // that claims to be committed and is not is worse than a slow save.
            mark = 0;
        }
        if mark > 0 {
            committed.fetch_max(mark, Ordering::Release);
        }
        // The barrier answers only once everything before it is synced, which
        // is what makes `drain` mean what a quit needs it to mean.
        for done in barriers {
            let _ = done.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pbd-saves-{name}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// The point of the whole module: the record is on disk when the mark says
    /// it is, and the caller never touched a file.
    #[test]
    fn a_record_is_on_disk_once_its_sequence_is_committed() {
        let slot = temporary("one");
        let mut writer = SaveWriter::new(&slot);
        let seq = writer.append("7 60 0\n".into());
        writer.drain();
        assert!(writer.committed() >= seq, "the mark reached the record");
        assert_eq!(writer.pending(), 0);
        let written = std::fs::read_to_string(slot.join(super::super::LOG)).unwrap();
        assert_eq!(written, "7 60 0\n");
        let _ = std::fs::remove_dir_all(&slot);
    }

    /// A burst is one fsync and one mark, and every line of it is there in the
    /// order it was sent. The order is the world's history, which is why this
    /// is a queue and not a pool of tasks.
    #[test]
    fn a_burst_lands_whole_and_in_order() {
        let slot = temporary("burst");
        let mut writer = SaveWriter::new(&slot);
        let mut last = 0;
        for cell in 0..64 {
            last = writer.append(format!("{cell} 60 0\n"));
        }
        writer.drain();
        assert!(writer.committed() >= last);
        let written = std::fs::read_to_string(slot.join(super::super::LOG)).unwrap();
        let cells: Vec<&str> = written
            .lines()
            .map(|line| line.split(' ').next().unwrap())
            .collect();
        assert_eq!(cells.len(), 64);
        assert_eq!(cells[0], "0");
        assert_eq!(cells[63], "63");
        let _ = std::fs::remove_dir_all(&slot);
    }

    /// A snapshot is replaced whole, so a reader never sees half of one.
    #[test]
    fn a_snapshot_is_replaced_rather_than_appended() {
        let slot = temporary("snap");
        let path = slot.join("world.ron");
        let mut writer = SaveWriter::new(&slot);
        writer.replace(path.clone(), "first");
        writer.replace(path.clone(), "second");
        writer.drain();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        assert!(
            !path.with_extension("tmp").exists(),
            "no litter left behind"
        );
        let _ = std::fs::remove_dir_all(&slot);
    }

    /// A write that cannot happen is REPORTED. A save that is silently not
    /// happening is the worst state this feature can be in, and the mark must
    /// not move for a record that never landed.
    #[test]
    fn a_write_that_fails_is_reported_and_does_not_move_the_mark() {
        let slot = temporary("bad");
        // A FILE where the slot directory should be: creating the directory
        // fails, so opening the log fails, for every job.
        std::fs::create_dir_all(slot.parent().unwrap()).unwrap();
        std::fs::write(&slot, "not a directory").unwrap();
        let mut writer = SaveWriter::new(&slot);
        let seq = writer.append("7 60 0\n".into());
        writer.drain();
        assert!(writer.failure().is_some(), "the failure is visible");
        assert!(writer.committed() < seq, "and nothing claims to be saved");
        assert_eq!(writer.pending(), 1);
        let _ = std::fs::remove_file(&slot);
    }

    /// The writer that does not write still answers every question, so a
    /// capture run needs no branch at the call site.
    #[test]
    fn a_writer_with_no_thread_saves_nothing_and_claims_nothing() {
        let mut writer = SaveWriter::none();
        writer.append("7 60 0\n".into());
        writer.drain();
        assert_eq!(writer.pending(), 0);
        assert!(writer.failure().is_none());
    }
}
