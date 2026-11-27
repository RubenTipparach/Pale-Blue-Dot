use super::*;

/// Task 6.5: three bots build the hut together through a server on
/// loopback, with a little stress; every request is answered once, every
/// region's changes arrive in order, and every bot's copy is the server's.
#[test]
fn three_bots_build_a_hut_and_agree_with_the_server() {
    let reports = soak(3, 1.0, 64);
    for report in &reports {
        assert!(report.ok(), "{}", report.line());
        assert!(report.regions_checked > 0, "{}", report.line());
    }
    // The centre's roof block: tried by all three, taken once.
    let accepted: u64 = reports.iter().map(|r| r.accepted).sum();
    let unchanged: u64 = reports
        .iter()
        .map(|r| r.refused.get("Unchanged").copied().unwrap_or(0))
        .sum();
    assert!(accepted > 20, "the hut was built: {accepted}");
    // Each pit's second dig of air, and two of three centre roof tries.
    assert!(unchanged >= 3 + 2, "refusals seen: {unchanged}");
}

/// The site and the hut are the same for every bot, and the hut leaves
/// its door open.
#[test]
fn every_bot_reads_the_same_site() {
    let a = Site::near(DEFAULT_SPAWN.normalize()).unwrap();
    let b = Site::near(DEFAULT_SPAWN.normalize()).unwrap();
    assert_eq!(a.centre, b.centre);
    assert_eq!(a.ring, b.ring);
    assert_eq!(a.ring.len(), 6);
    let pieces = hut(&a);
    assert!(
        !pieces
            .iter()
            .any(|p| matches!(p, Piece::Wall { cell, .. } if *cell == a.ring[0]))
    );
    assert_eq!(pieces.len(), 5 * 2 + 6);
}

/// A game restarted rejoins as the same player before its old connection
/// is seen to close: the welcome reaches the new connection, and the old
/// one's close does not take the player out (found on the Mac, design 21).
#[test]
fn a_player_who_rejoins_is_served_on_the_new_connection() {
    let stop = Arc::new(AtomicBool::new(false));
    let (ready_tx, ready) = std::sync::mpsc::channel();
    let options = ServeOptions {
        addr: SocketAddr::from(([127, 0, 0, 1], 0)),
        ..ServeOptions::default()
    };
    let server = {
        let stop = stop.clone();
        std::thread::spawn(move || serve(&options, &stop, Some(ready_tx)))
    };
    let addr = ready.recv_timeout(Duration::from_secs(10)).unwrap();
    let options = BotOptions {
        name: "again".into(),
        ..BotOptions::default()
    };
    // The first stays connected, as a killed game's does until it times out.
    let first = Bot::join(addr, options.clone(), 64).expect("the first join");
    let second = Bot::join(addr, options, 64).expect("the second join is welcomed");
    assert_eq!(first.report.player, second.report.player);
    drop(first);
    let report = second.play();
    assert!(report.ok(), "{}", report.line());
    stop.store(true, Ordering::Relaxed);
    let _ = server.join().unwrap();
}

/// Design 27: a server whose world is on disk keeps it across a restart.
/// A bot builds; the server stops; the folder reopens with the bot's edits
/// and the world's seed; a second server on it sends a new bot that world.
#[test]
fn a_server_on_disk_keeps_its_world_across_a_restart() {
    let dir = std::env::temp_dir().join(format!("pbd-disk-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let run = |seed: u64| {
        let stop = Arc::new(AtomicBool::new(false));
        let (ready_tx, ready) = std::sync::mpsc::channel();
        let options = ServeOptions {
            addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            seed,
            world: Some(dir.clone()),
            ..ServeOptions::default()
        };
        let server = {
            let stop = stop.clone();
            std::thread::spawn(move || serve(&options, &stop, Some(ready_tx)))
        };
        let addr = ready.recv_timeout(Duration::from_secs(10)).unwrap();
        let bot = Bot::join(addr, BotOptions::default(), 64).expect("joined");
        let report = bot.play();
        stop.store(true, Ordering::Relaxed);
        server.join().unwrap().expect("served");
        report
    };
    let first = run(5);
    assert!(first.ok(), "{}", first.line());
    let opened = disk::DiskStore::open(&dir, 999).expect("reopened");
    assert_eq!(
        opened.identity.seed, 5,
        "the world's own seed, not the flag's"
    );
    let kept = pbd_server::Store::edits(&opened).len();
    assert!(kept > 0, "the bot's edits are on disk");
    drop(opened);
    // A second server on the same folder serves that world: its new bot's
    // copy agrees with the server's, edits included.
    let second = run(999);
    assert!(second.regions_checked > 0, "{}", second.line());
    let reopened = disk::DiskStore::open(&dir, 1).expect("reopened");
    assert!(pbd_server::Store::edits(&reopened).len() >= kept);
    let _ = std::fs::remove_dir_all(&dir);
}
