#[cfg(feature = "desktop")]
mod desktop;
mod headless;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|s| s == "--headless" || s.parse::<u32>().is_ok())
    {
        let index = usize::from(args[0] == "--headless");
        let steps = args
            .get(index)
            .map(|s| s.parse::<u32>().expect("steps must be an integer"))
            .unwrap_or(600);
        assert!(
            (1..=360_000).contains(&steps),
            "steps must be between 1 and 360000"
        );
        headless::run(steps);
        return;
    }
    if args.first().is_some_and(|s| s == "--colliding-pairs") {
        colliding_pairs(&args[1..]);
        return;
    }
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!(
            "Pale Blue Dot - planet explorer\n\n  run.bat                         Walk on the planet\n  run.bat --fly                   Start in free flight\n  run.bat --tour                  Automated planet flight\n  run.bat --verify-flight         Headless full circumnavigation check\n  run.bat --headless 600          Core simulation smoke\n  run.bat --capture walk.png --walk --frames 180\n  run.bat --capture tern.png --aboard tern --frames 180\n  run.bat --capture seat.png --aboard kestrel --seat\n  run.bat --capture offset.png --aboard loon --render-offset 8192 -4096 2048\n\nVehicle capture: --aboard kestrel|tern|loon; --seat selects the seat view.\nStatic capture: --render-offset x y z translates the planet in metres.\n  run.bat --capture orbit.png --view orbit --frames 180\n  run.bat --capture sea.png --view shore --height 50\n  run.bat --capture rain.png --view shore --rain 1\n  run.bat --capture cave.png --view cave\n  run.bat --capture pause.png --menu pause    Photograph a menu screen\n  run.bat --walk --spawn mouth        Start beside a cave mouth\n  run.bat --world Caves               Open (or make) a named world\n\nViews: orbit, coast, surface, night, pole, shore (eye-height polar coast), wade, dive,\n       cave (inside a generated cave), overhang (looking up at its roof),\n       mouth (a tunnel opening seen from outside; --spawn mouth anchors the world on one)\n\nControls. The same table the settings page draws, so the two cannot disagree:"
        );
        // One table, read here and by the settings page. It used to be written
        // out three times - here, in a panel behind `H`, and in a strip over
        // the hotbar - and all three had gone stale in the same way: none of
        // them mentioned digging, months after it shipped.
        #[cfg(feature = "desktop")]
        println!("{}", pbd_app::controls::help_text());
        return;
    }
    #[cfg(feature = "desktop")]
    desktop::run(&args);
    #[cfg(not(feature = "desktop"))]
    panic!("Desktop feature disabled. Use --headless [steps] or build default features.");
}

/// `--colliding-pairs [count]`: the finest cells whose old hashes collide,
/// nearest the default spawn first (`exact-cell-keys` task 1.2). Before exact
/// keys, digging either cell of a pair also dug the other.
fn colliding_pairs(args: &[String]) {
    use pbd_app::planet::PLANET_RADIUS;
    use pbd_core::planet_gen::TerrainConfig;
    let count = args.first().and_then(|s| s.parse().ok()).unwrap_or(10);
    let spawn = bevy::math::Vec3::new(0.8776, 0.4794, 0.0).normalize();
    println!("hash        key        metres from spawn  ground m  direction");
    for pair in pbd_app::saves::migrate::colliding_pairs_near(spawn, count) {
        for (index, (key, at)) in pair.cells.iter().enumerate() {
            let metres = at.dot(spawn).clamp(-1.0, 1.0).acos() * PLANET_RADIUS;
            let ground = pbd_core::column::surface_m(&TerrainConfig::TENEBRIS, *at);
            let hash = if index == 0 {
                pair.hash.to_string()
            } else {
                String::new()
            };
            println!(
                "{hash:<11} {key:<10} {metres:>17.0} {ground:>9.0}  {:.6} {:.6} {:.6}",
                at.x, at.y, at.z
            );
        }
    }
}
