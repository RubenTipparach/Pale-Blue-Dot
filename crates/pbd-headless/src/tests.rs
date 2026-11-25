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
