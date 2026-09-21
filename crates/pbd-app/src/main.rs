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
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!(
            "Pale Blue Dot - planet explorer\n\n  run.bat                         Walk on the planet\n  run.bat --fly                   Start in free flight\n  run.bat --tour                  Automated planet flight\n  run.bat --verify-flight         Headless full circumnavigation check\n  run.bat --headless 600          Core simulation smoke\n  run.bat --capture walk.png --walk --frames 180\n  run.bat --capture orbit.png --view orbit --frames 180\n  run.bat --capture sea.png --view shore --height 50\n  run.bat --capture rain.png --view shore --rain 1\n  run.bat --capture cave.png --view cave\n  run.bat --capture pause.png --menu pause    Photograph a menu screen\n  run.bat --walk --spawn mouth        Start beside a cave mouth\n  run.bat --world Caves               Open (or make) a named world\n\nViews: orbit, coast, surface, night, pole, shore (eye-height polar coast), wade, dive,\n       cave (inside a generated cave), overhang (looking up at its roof),\n       mouth (a tunnel opening seen from outside; --spawn mouth anchors the world on one)\n\nControls. The same table the settings page draws, so the two cannot disagree:"
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
