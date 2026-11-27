//! `pbd-headless`: a dedicated server and protocol bots with no window
//! (`multiplayer` design 21).
//!
//! ```text
//! pbd-headless serve [--port 47124] [--bind 127.0.0.1] [--seed 77] [--clock 1200] [--kit 64] [--seconds S] [--world DIR]
//! pbd-headless bot --join 127.0.0.1:47124 [--name bot0] [--role 0/3] [--at LAT,LON] [--stress S] [--rate HZ] [--pace S] [--kit 64]
//! pbd-headless soak [--bots 3] [--stress S] [--kit 64]
//! ```
//!
//! locale: dev

use pbd_headless::{Bot, BotOptions, ServeOptions, serve, soak};
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;

fn value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn number<T: std::str::FromStr>(args: &[String], flag: &str, default: T) -> T {
    match value(args, flag) {
        Some(v) => v
            .parse()
            .unwrap_or_else(|_| panic!("{flag} takes a number, not {v:?}")),
        None => default,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("serve") => {
            let bind = value(&args, "--bind").unwrap_or("127.0.0.1");
            let port: u16 = number(&args, "--port", 47124);
            let options = ServeOptions {
                // A name resolves too: Fly's UDP is bound on
                // `fly-global-services` (`multiplayer` design 27).
                addr: std::net::ToSocketAddrs::to_socket_addrs(&(bind, port))
                    .ok()
                    .and_then(|mut a| a.next())
                    .expect("--bind takes an address or a host name"),
                seed: number(&args, "--seed", 77),
                clock_s: number(&args, "--clock", 1200.0),
                kit: number(&args, "--kit", 64),
                seconds: value(&args, "--seconds").map(|s| s.parse().expect("--seconds")),
                world: value(&args, "--world").map(std::path::PathBuf::from),
            };
            match serve(&options, &AtomicBool::new(false), None) {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("could not host: {e}");
                    1
                }
            }
        }
        Some("bot") => {
            let addr: SocketAddr = value(&args, "--join")
                .expect("bot needs --join <address:port>")
                .parse()
                .expect("--join takes address:port");
            let (role, of) = value(&args, "--role")
                .and_then(|r| r.split_once('/'))
                .map(|(a, b)| {
                    (
                        a.parse().expect("--role i/n"),
                        b.parse().expect("--role i/n"),
                    )
                })
                .unwrap_or((0, 1));
            let near = value(&args, "--at")
                .and_then(|s| s.split_once(','))
                .map(|(lat, lon)| {
                    pbd_core::geo::direction(pbd_core::geo::LatLon {
                        lat: lat
                            .trim()
                            .parse::<f32>()
                            .expect("--at LAT,LON")
                            .to_radians(),
                        lon: lon
                            .trim()
                            .parse::<f32>()
                            .expect("--at LAT,LON")
                            .to_radians(),
                    })
                })
                .unwrap_or(pbd_headless::DEFAULT_SPAWN.normalize());
            let options = BotOptions {
                name: value(&args, "--name").unwrap_or("bot0").into(),
                role,
                of,
                near,
                stress_s: number(&args, "--stress", 0.0),
                rate_hz: number(&args, "--rate", 4.0),
                pace_s: number(&args, "--pace", 0.0),
            };
            match Bot::join(addr, options, number(&args, "--kit", 64)) {
                Ok(bot) => {
                    let report = bot.play();
                    println!("{}", report.line());
                    i32::from(!report.ok())
                }
                Err(why) => {
                    println!("BOT_FAIL could not join {addr}: {why}");
                    1
                }
            }
        }
        Some("soak") => {
            let reports = soak(
                number(&args, "--bots", 3),
                number(&args, "--stress", 10.0),
                number(&args, "--kit", 64),
            );
            for r in &reports {
                println!("{}", r.line());
            }
            i32::from(!reports.iter().all(|r| r.ok()))
        }
        _ => {
            eprintln!(
                "pbd-headless serve|bot|soak (see the top of crates/pbd-headless/src/main.rs)"
            );
            2
        }
    };
    std::process::exit(code);
}
