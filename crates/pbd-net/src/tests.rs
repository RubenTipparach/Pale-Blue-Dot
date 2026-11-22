use super::*;
use pbd_core::inventory::Slots;
use pbd_core::net::protocol::{JoinRefusal, PROTOCOL, PROTOCOL_MIN, Versions};
use pbd_core::terrain::{GENERATOR_VERSION, TOPOLOGY_VERSION};
use std::time::{Duration, Instant};

fn versions(world: u64, generator: u32) -> Versions {
    Versions {
        protocol: PROTOCOL,
        protocol_min: PROTOCOL_MIN,
        topology: TOPOLOGY_VERSION,
        generator,
        content: 0,
        world,
    }
}

fn hello(world: u64, generator: u32) -> ToServer {
    ToServer::Hello {
        versions: versions(world, generator),
        key: [5; 16],
        name: "guest".into(),
    }
}

/// Poll until `done` says so, or panic after a few seconds.
fn until<T>(what: &str, mut poll: impl FnMut() -> Option<T>) -> T {
    let start = Instant::now();
    loop {
        if let Some(t) = poll() {
            return t;
        }
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "timed out: {what}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A server on a host: each Opened connection's first message is its join.
fn serve(host: &Host, server: &mut pbd_server::Server) -> Vec<HostEvent> {
    let events = host.poll();
    for event in &events {
        if let HostEvent::Message(connection, hello @ ToServer::Hello { .. }) = event
            && server.join(*connection, hello, Slots::default()).is_none()
        {
            for (_, message) in server.drain() {
                host.send(*connection, &message);
            }
        }
    }
    for (to, message) in server.drain() {
        let connection = match to {
            pbd_server::To::Player(p) => u64::from(p),
            pbd_server::To::Connection(c) => c,
        };
        host.send(connection, &message);
    }
    events
}

fn server() -> pbd_server::Server {
    pbd_server::Server::new(
        pbd_server::World {
            terrain: pbd_core::planet_gen::TerrainConfig::for_version(GENERATOR_VERSION).unwrap(),
            worms: pbd_core::worms::WormField::DEFAULT,
            versions: versions(77, GENERATOR_VERSION),
        },
        10.0,
    )
}

/// Task 6.1: two ends on loopback connect, a join is answered with the
/// world to open, and messages go both ways in order.
#[test]
fn a_client_joins_a_host_on_loopback() {
    let host = Host::bind_to("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut server = server();
    let client = Client::connect(host.local_addr(), Duration::from_secs(10)).unwrap();
    client.send(&hello(0, GENERATOR_VERSION));
    let welcome = until("the welcome", || {
        serve(&host, &mut server);
        client.poll().into_iter().next()
    });
    assert_eq!(
        welcome,
        Ok(ToClient::Welcome {
            player: 0,
            clock_s: 10.0,
            seed: 77,
            generator: GENERATOR_VERSION,
            topology: TOPOLOGY_VERSION,
        })
    );
    // Many messages, in order, both ways.
    for n in 0..200u32 {
        client.send(&ToServer::Resync { region: n });
    }
    let mut got = Vec::new();
    until("every message", || {
        for event in host.poll() {
            if let HostEvent::Message(c, ToServer::Resync { region }) = event {
                got.push(region);
                host.send(
                    c,
                    &ToClient::Clock {
                        clock_s: f64::from(region),
                    },
                );
            }
        }
        (got.len() == 200).then_some(())
    });
    assert_eq!(got, (0..200).collect::<Vec<_>>());
    let mut back = Vec::new();
    until("every reply", || {
        back.extend(client.poll());
        (back.len() == 200).then_some(())
    });
    assert_eq!(back[199], Ok(ToClient::Clock { clock_s: 199.0 }));
}

/// Task 6.1: a build of another generator is refused with its plain
/// message, and nothing else follows.
#[test]
fn a_mismatched_generator_is_refused() {
    let host = Host::bind_to("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut server = server();
    let client = Client::connect(host.local_addr(), Duration::from_secs(10)).unwrap();
    client.send(&hello(0, GENERATOR_VERSION - 1));
    let answer = until("the refusal", || {
        serve(&host, &mut server);
        client.poll().into_iter().next()
    });
    assert_eq!(
        answer,
        Ok(ToClient::Rejected {
            reason: JoinRefusal::WorldVersion
        })
    );
}

/// Bytes that are not a message close the connection, and the host says
/// which.
#[test]
fn bytes_that_are_not_a_message_close_the_connection() {
    let host = Host::bind_to("127.0.0.1:0".parse().unwrap()).unwrap();
    let client = Client::connect(host.local_addr(), Duration::from_secs(10)).unwrap();
    let _ = client.out.send(vec![250, 1, 2]);
    let seen = until("the unreadable message", || {
        host.poll()
            .into_iter()
            .find(|e| matches!(e, HostEvent::Unreadable(..)))
    });
    assert_eq!(seen, HostEvent::Unreadable(0, Decode::Unknown(250)));
    until("the close", || client.is_closed().then_some(()));
}

/// Nobody listening: the connect gives up within its time.
#[test]
fn a_host_that_is_not_there_times_out() {
    let free = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = free.local_addr().unwrap();
    drop(free);
    let started = Instant::now();
    assert!(Client::connect(addr, Duration::from_millis(500)).is_err());
    assert!(started.elapsed() < Duration::from_secs(5));
}
