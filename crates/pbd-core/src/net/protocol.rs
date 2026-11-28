//! The protocol's messages and their bytes (`multiplayer` design 6, 9 and
//! 15, task 3.2).
//!
//! Encoded by hand with the save format's discipline: little-endian, every
//! list and string length-prefixed, a tag byte first. A message is one
//! transport frame, so the decoder takes exactly one message's bytes and
//! refuses trailing ones. A tag this build does not know is refused as
//! [`Decode::Unknown`], never guessed at, so a newer peer's message is a
//! clean error.
//!
//! Positions go as body-relative `f64` (design 7); nothing on the wire is an
//! `f32` relative to a render origin.

use crate::accept::PackMove;
use crate::terrain::{Material, material_code, material_of};
use crate::world_frame::{FrameId, WorldPos};
use glam::DVec3;

/// This build's protocol. 2: `ToClient::Slots` (design 22), `Drop` and
/// `DropLeft` (design 23).
pub const PROTOCOL: u16 = 2;
/// The oldest protocol this build still speaks (design 15.9). A version-1
/// game cannot read its own slots, so it is refused rather than left with
/// an empty hotbar (design 22).
pub const PROTOCOL_MIN: u16 = 2;
/// The longest chat line or name, in bytes.
pub const MAX_TEXT: usize = 512;
/// The most items any one list may hold, which bounds what a malformed
/// length can make a decoder allocate.
pub const MAX_LIST: usize = 1 << 16;
/// The largest snapshot, in bytes.
pub const MAX_SNAPSHOT: usize = 1 << 22;
/// The longest record body, bytes (design 26).
pub const MAX_RECORD: usize = 1 << 16;

/// What a world and a build must agree on before anything else is sent
/// (design 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Versions {
    /// The newest protocol the sender speaks, and the oldest.
    pub protocol: u16,
    pub protocol_min: u16,
    /// Exact matches: they decide the world itself.
    pub topology: u32,
    pub generator: u32,
    /// The digest of the item, recipe and block tables.
    pub content: u64,
    /// The world's identity.
    pub world: u64,
}

/// Why a join was refused. Each has a plain line in the locale file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinRefusal {
    /// No protocol both sides speak.
    GameVersion,
    /// The topology, generator or content versions differ.
    WorldVersion,
    /// A different world.
    DifferentWorld,
    Full,
    Banned,
    WrongPassword,
}

/// What a player tried to do: what the client sends, never the result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Dig the layer of the cell.
    Dig { cell: u32, layer: u16 },
    /// Place what the hotbar's slot holds into the layer of the cell.
    Place { cell: u32, layer: u16, slot: u8 },
    /// A move in the pack (design 19).
    Pack(crate::accept::PackMove),
    /// Pick up a drop the client's magnet brought in (design 19).
    Pick { drop: u64 },
    /// Toss `count` of what `slot` holds, to land at `at_mm`, whole
    /// millimetres in the body's frame (design 25).
    Toss {
        slot: u8,
        count: u16,
        at_mm: [i64; 3],
    },
    /// Sow the seed in `slot` on the tilled soil at the cell's layer
    /// (design 26).
    Plant { cell: u32, layer: u16, slot: u8 },
    /// Harvest the crop on the cell.
    Harvest { cell: u32 },
}

/// Why a request was refused. No undo rides with it: the server re-sends
/// the touched cells and the whole inventory (design 15.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The layer is bedrock or above the world.
    OutOfWorld,
    /// The cell already holds that.
    Unchanged,
    /// Nothing in the slot to place.
    EmptySlot,
    /// One lamp to a column.
    LampTaken,
    /// Too far, too soon, or not the tool (design 15.7).
    Cheat,
    /// The save did not take it.
    NotSaved,
    /// The drop is not there: picked up already, or never was.
    Gone,
    /// Farming (design 26): the ground is not tilled soil with air over it.
    NotTilled,
    /// A crop already grows there.
    Growing,
    /// The crop does not grow in this climate.
    Climate,
    /// Nothing growing there.
    NoCrop,
    /// Not ripe yet.
    NotRipe,
    /// No room in the pack for all of it.
    NoRoom,
}

impl From<crate::farming::accept::Refused> for Refusal {
    fn from(r: crate::farming::accept::Refused) -> Self {
        use crate::farming::accept::Refused as F;
        match r {
            F::NotTilled => Refusal::NotTilled,
            F::Growing => Refusal::Growing,
            F::Climate => Refusal::Climate,
            F::EmptySlot => Refusal::EmptySlot,
            F::NoCrop => Refusal::NoCrop,
            F::NotRipe => Refusal::NotRipe,
            F::NoRoom => Refusal::NoRoom,
        }
    }
}

/// A change to a region's state, in the server's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Edit {
        cell: u32,
        layer: u16,
        material: Material,
    },
}

/// A far interaction between two players (design 12).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transaction {
    /// Give `count` of what the giver's slot holds.
    Give { slot: u8, count: u16 },
    /// A marker on the other's map, at a cell.
    Mark { cell: u32 },
}

/// A player's place and look.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub at: WorldPos,
    pub yaw: f32,
    pub pitch: f32,
}

/// A fixed tangent frame at `up` (a unit vector from the body's centre),
/// east then north, that every game works yaw in (design 24): east is the
/// body's Z axis crossed with up, or its Y axis near the poles of Z.
pub fn tangent_frame(up: DVec3) -> (DVec3, DVec3) {
    let axis = if up.z.abs() < 0.9 { DVec3::Z } else { DVec3::Y };
    let east = axis.cross(up).normalize();
    let north = up.cross(east);
    (east, north)
}

/// The yaw of `forward` at `up`, radians from north toward east, in the
/// plane tangent there (design 24).
pub fn yaw_of(forward: DVec3, up: DVec3) -> f32 {
    let (east, north) = tangent_frame(up);
    forward.dot(east).atan2(forward.dot(north)) as f32
}

/// The unit tangent at `up` that faces `yaw` (the inverse of [`yaw_of`]).
pub fn forward_of(yaw: f32, up: DVec3) -> DVec3 {
    let (east, north) = tangent_frame(up);
    let yaw = f64::from(yaw);
    north * yaw.cos() + east * yaw.sin()
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToServer {
    /// The first message: the versions, the install's key and a name.
    Hello {
        versions: Versions,
        key: [u8; 16],
        name: String,
    },
    /// A request, numbered by the client.
    Request {
        id: u32,
        request: Request,
    },
    /// How far the client's tier reaches, m, and the regions it dropped
    /// (design 15.5).
    Reach {
        reach_m: f32,
        dropped: Vec<u32>,
    },
    /// Snapshots received (design 15.4).
    Got {
        regions: Vec<u32>,
    },
    /// A region whose digest did not match: send it whole.
    Resync {
        region: u32,
    },
    /// Where the player is, 20 times a second.
    Presence {
        pose: Pose,
    },
    /// A far interaction with another player.
    Transaction {
        id: u32,
        with: u32,
        transaction: Transaction,
    },
    Chat {
        text: String,
    },
    /// The player's own character (`player-character` 4.3), sent after the
    /// hello and whenever it changes.
    Look {
        look: crate::townsfolk::look::Look,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToClient {
    /// Joined: the player's number, the world clock, s, and the world to
    /// open, its seed, generator and topology (design 20), for a client
    /// that joined without it.
    Welcome {
        player: u32,
        clock_s: f64,
        seed: u64,
        generator: u32,
        topology: u32,
    },
    /// Not joined, and nothing else follows.
    Rejected {
        reason: JoinRefusal,
    },
    /// The request was taken, and its change is `seq` in `region`.
    Accepted {
        id: u32,
        region: u32,
        seq: u64,
    },
    Refused {
        id: u32,
        reason: Refusal,
    },
    /// A change, `seq` in `region`.
    Change {
        region: u32,
        seq: u64,
        change: Change,
    },
    /// A region whole, its canonical bytes (`digest::encode`) as of `seq`.
    Snapshot {
        region: u32,
        seq: u64,
        bytes: Vec<u8>,
    },
    /// A region's digest as of `seq`.
    Digest {
        region: u32,
        seq: u64,
        digest: u64,
    },
    /// Another player's pose.
    Presence {
        player: u32,
        pose: Pose,
    },
    /// The world clock, s, every 5 s (design 15.8).
    Clock {
        clock_s: f64,
    },
    /// A transaction's outcome, to each party.
    Settled {
        id: u32,
        done: bool,
    },
    Chat {
        from: u32,
        text: String,
    },
    /// The player's own carried slots, whole (design 22): after the
    /// welcome, after a request that changed them, after a refusal. Which
    /// slot is selected is the player's own and is not sent.
    Slots {
        slots: crate::inventory::Slots,
    },
    /// A drop a dig made, whoever dug (design 23).
    Drop {
        drop: crate::drops::ItemDrop,
    },
    /// A pickup was decided: `left` remain on drop `id`, 0 for gone.
    DropLeft {
        id: u64,
        left: u16,
    },
    /// A player left the world (design 24).
    Gone {
        player: u32,
    },
    /// A world record the server's store took (design 26).
    Record {
        record: crate::records::Record,
    },
    /// Another player's character (`player-character` 4.3).
    Look {
        player: u32,
        look: crate::townsfolk::look::Look,
    },
}

/// Why bytes did not decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decode {
    /// Ran out before the message did.
    Short,
    /// A tag this build does not know: a newer peer.
    Unknown(u8),
    /// A value no message has: a bad material, a non-finite number, text
    /// that is not UTF-8, a list past its bound.
    Invalid,
    /// Bytes left after the message.
    Trailing,
}

/// Whether two builds can play together, and the protocol they use
/// (design 9, 15.9): the newest both speak, and every version that decides
/// the world equal.
pub fn agree(server: &Versions, client: &Versions) -> Result<u16, JoinRefusal> {
    let protocol = server.protocol.min(client.protocol);
    if protocol < server.protocol_min.max(client.protocol_min) {
        return Err(JoinRefusal::GameVersion);
    }
    // A client joining a world it does not hold (world 0, design 20) takes
    // the world's generator from the welcome, and refuses it itself if it
    // does not carry it; the topology and content are this build's and must
    // match.
    let generator_matches = client.world == 0 || server.generator == client.generator;
    if (server.topology, server.content) != (client.topology, client.content) || !generator_matches
    {
        return Err(JoinRefusal::WorldVersion);
    }
    // World 0 is "any": a player joining a world they do not hold yet
    // (design 20), who is sent it in the welcome.
    if client.world != 0 && server.world != client.world {
        return Err(JoinRefusal::DifferentWorld);
    }
    Ok(protocol)
}

struct W(Vec<u8>);

impl W {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f64(&mut self, v: f64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.u32(v.len() as u32);
        self.0.extend_from_slice(v);
    }
    /// A slot (design 22): a count, 0 for empty, then a kind byte and the
    /// item's code; a good by its code, never its table index.
    fn stack(&mut self, stack: Option<crate::inventory::Stack>) {
        use crate::inventory::Item;
        let Some(stack) = stack.filter(|s| s.count > 0) else {
            self.u16(0);
            return;
        };
        self.u16(stack.count);
        match stack.item {
            Item::Block(material) => {
                self.u8(1);
                self.u8(material_code(material));
            }
            Item::Tool(tool) => {
                self.u8(2);
                self.u8(tool.index() as u8);
            }
            Item::Fish(species) => {
                self.u8(3);
                self.u16(species);
            }
            Item::Good(good) => {
                self.u8(4);
                self.bytes(good.code().as_bytes());
            }
        }
    }
    fn regions(&mut self, v: &[u32]) {
        self.u32(v.len() as u32);
        for &r in v {
            self.u32(r);
        }
    }
    fn versions(&mut self, v: &Versions) {
        self.u16(v.protocol);
        self.u16(v.protocol_min);
        self.u32(v.topology);
        self.u32(v.generator);
        self.u64(v.content);
        self.u64(v.world);
    }
    fn pose(&mut self, p: &Pose) {
        self.u16(p.at.frame.0);
        for c in p.at.pos.to_array() {
            self.f64(c);
        }
        self.f32(p.yaw);
        self.f32(p.pitch);
    }
}

struct R<'a>(&'a [u8]);

impl<'a> R<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Decode> {
        if self.0.len() < n {
            return Err(Decode::Short);
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }
    fn u8(&mut self) -> Result<u8, Decode> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, Decode> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, Decode> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, Decode> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, Decode> {
        let v = f32::from_le_bytes(self.take(4)?.try_into().unwrap());
        v.is_finite().then_some(v).ok_or(Decode::Invalid)
    }
    fn f64(&mut self) -> Result<f64, Decode> {
        let v = f64::from_le_bytes(self.take(8)?.try_into().unwrap());
        v.is_finite().then_some(v).ok_or(Decode::Invalid)
    }
    fn len(&mut self, max: usize, each: usize) -> Result<usize, Decode> {
        let n = self.u32()? as usize;
        if n > max {
            return Err(Decode::Invalid);
        }
        if self.0.len() < n.saturating_mul(each) {
            return Err(Decode::Short);
        }
        Ok(n)
    }
    fn bytes(&mut self, max: usize) -> Result<Vec<u8>, Decode> {
        let n = self.len(max, 1)?;
        Ok(self.take(n)?.to_vec())
    }
    fn text(&mut self) -> Result<String, Decode> {
        String::from_utf8(self.bytes(MAX_TEXT)?).map_err(|_| Decode::Invalid)
    }
    fn regions(&mut self) -> Result<Vec<u32>, Decode> {
        let n = self.len(MAX_LIST, 4)?;
        (0..n).map(|_| self.u32()).collect()
    }
    fn versions(&mut self) -> Result<Versions, Decode> {
        Ok(Versions {
            protocol: self.u16()?,
            protocol_min: self.u16()?,
            topology: self.u32()?,
            generator: self.u32()?,
            content: self.u64()?,
            world: self.u64()?,
        })
    }
    fn pose(&mut self) -> Result<Pose, Decode> {
        let frame = FrameId(self.u16()?);
        let pos = DVec3::new(self.f64()?, self.f64()?, self.f64()?);
        Ok(Pose {
            at: WorldPos::new(frame, pos),
            yaw: self.f32()?,
            pitch: self.f32()?,
        })
    }
    /// A character's look (`player-character`): refused unless whole and
    /// one the townsfolk can draw.
    fn look(&mut self) -> Result<crate::townsfolk::look::Look, Decode> {
        let bytes = self.take(crate::townsfolk::character::ENCODED_LEN)?;
        crate::townsfolk::character::decode(bytes).ok_or(Decode::Invalid)
    }
    fn material(&mut self) -> Result<Material, Decode> {
        material_of(self.u8()?).ok_or(Decode::Invalid)
    }
    /// A slot (design 22): a count, 0 for empty, then its item.
    fn stack(&mut self) -> Result<Option<crate::inventory::Stack>, Decode> {
        use crate::inventory::{Item, Stack, Tool};
        let count = self.u16()?;
        if count == 0 {
            return Ok(None);
        }
        let item = match self.u8()? {
            1 => Item::Block(self.material()?),
            2 => Item::Tool(*Tool::ALL.get(self.u8()? as usize).ok_or(Decode::Invalid)?),
            3 => Item::Fish(self.u16()?),
            4 => Item::Good(crate::goods::Good::by_code(&self.text()?).ok_or(Decode::Invalid)?),
            _ => return Err(Decode::Invalid),
        };
        if count > item.stack_limit() {
            return Err(Decode::Invalid);
        }
        Ok(Some(Stack::new(item, count)))
    }
    fn end<T>(self, value: T) -> Result<T, Decode> {
        if self.0.is_empty() {
            Ok(value)
        } else {
            Err(Decode::Trailing)
        }
    }
}

const JOIN_REFUSALS: [JoinRefusal; 6] = [
    JoinRefusal::GameVersion,
    JoinRefusal::WorldVersion,
    JoinRefusal::DifferentWorld,
    JoinRefusal::Full,
    JoinRefusal::Banned,
    JoinRefusal::WrongPassword,
];
const REFUSALS: [Refusal; 13] = [
    Refusal::OutOfWorld,
    Refusal::Unchanged,
    Refusal::EmptySlot,
    Refusal::LampTaken,
    Refusal::Cheat,
    Refusal::NotSaved,
    Refusal::Gone,
    Refusal::NotTilled,
    Refusal::Growing,
    Refusal::Climate,
    Refusal::NoCrop,
    Refusal::NotRipe,
    Refusal::NoRoom,
];

fn code_of<T: PartialEq>(table: &[T], v: &T) -> u8 {
    table
        .iter()
        .position(|t| t == v)
        .expect("every value is in its table") as u8
}

fn of_code<T: Copy>(table: &[T], code: u8) -> Result<T, Decode> {
    table.get(code as usize).copied().ok_or(Decode::Invalid)
}

impl ToServer {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = W(Vec::new());
        match self {
            Self::Hello {
                versions,
                key,
                name,
            } => {
                w.u8(1);
                w.versions(versions);
                w.0.extend_from_slice(key);
                w.bytes(name.as_bytes());
            }
            Self::Request { id, request } => {
                w.u8(2);
                w.u32(*id);
                match *request {
                    Request::Dig { cell, layer } => {
                        w.u8(1);
                        w.u32(cell);
                        w.u16(layer);
                    }
                    Request::Place { cell, layer, slot } => {
                        w.u8(2);
                        w.u32(cell);
                        w.u16(layer);
                        w.u8(slot);
                    }
                    Request::Pack(mv) => {
                        w.u8(3);
                        match mv {
                            PackMove::Shift { from, to, count } => {
                                w.u8(1);
                                w.u8(from as u8);
                                w.u8(to as u8);
                                w.u16(count);
                            }
                            PackMove::Send { index } => {
                                w.u8(2);
                                w.u8(index as u8);
                            }
                            PackMove::Trash { index, count } => {
                                w.u8(3);
                                w.u8(index as u8);
                                w.u16(count);
                            }
                        }
                    }
                    Request::Pick { drop } => {
                        w.u8(4);
                        w.u64(drop);
                    }
                    Request::Plant { cell, layer, slot } => {
                        w.u8(6);
                        w.u32(cell);
                        w.u16(layer);
                        w.u8(slot);
                    }
                    Request::Harvest { cell } => {
                        w.u8(7);
                        w.u32(cell);
                    }
                    Request::Toss { slot, count, at_mm } => {
                        w.u8(5);
                        w.u8(slot);
                        w.u16(count);
                        for v in at_mm {
                            w.u64(v as u64);
                        }
                    }
                }
            }
            Self::Reach { reach_m, dropped } => {
                w.u8(3);
                w.f32(*reach_m);
                w.regions(dropped);
            }
            Self::Got { regions } => {
                w.u8(4);
                w.regions(regions);
            }
            Self::Resync { region } => {
                w.u8(5);
                w.u32(*region);
            }
            Self::Presence { pose } => {
                w.u8(6);
                w.pose(pose);
            }
            Self::Transaction {
                id,
                with,
                transaction,
            } => {
                w.u8(7);
                w.u32(*id);
                w.u32(*with);
                match *transaction {
                    Transaction::Give { slot, count } => {
                        w.u8(1);
                        w.u8(slot);
                        w.u16(count);
                    }
                    Transaction::Mark { cell } => {
                        w.u8(2);
                        w.u32(cell);
                    }
                }
            }
            Self::Chat { text } => {
                w.u8(8);
                w.bytes(text.as_bytes());
            }
            Self::Look { look } => {
                w.u8(9);
                crate::townsfolk::character::encode(look, &mut w.0);
            }
        }
        w.0
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Decode> {
        let mut r = R(bytes);
        let message = match r.u8()? {
            1 => {
                let versions = r.versions()?;
                let key: [u8; 16] = r.take(16)?.try_into().unwrap();
                Self::Hello {
                    versions,
                    key,
                    name: r.text()?,
                }
            }
            2 => {
                let id = r.u32()?;
                let request = match r.u8()? {
                    1 => Request::Dig {
                        cell: r.u32()?,
                        layer: r.u16()?,
                    },
                    2 => Request::Place {
                        cell: r.u32()?,
                        layer: r.u16()?,
                        slot: r.u8()?,
                    },
                    3 => Request::Pack(match r.u8()? {
                        1 => PackMove::Shift {
                            from: usize::from(r.u8()?),
                            to: usize::from(r.u8()?),
                            count: r.u16()?,
                        },
                        2 => PackMove::Send {
                            index: usize::from(r.u8()?),
                        },
                        3 => PackMove::Trash {
                            index: usize::from(r.u8()?),
                            count: r.u16()?,
                        },
                        other => return Err(Decode::Unknown(other)),
                    }),
                    4 => Request::Pick { drop: r.u64()? },
                    6 => Request::Plant {
                        cell: r.u32()?,
                        layer: r.u16()?,
                        slot: r.u8()?,
                    },
                    7 => Request::Harvest { cell: r.u32()? },
                    5 => Request::Toss {
                        slot: r.u8()?,
                        count: r.u16()?,
                        at_mm: [r.u64()? as i64, r.u64()? as i64, r.u64()? as i64],
                    },
                    other => return Err(Decode::Unknown(other)),
                };
                Self::Request { id, request }
            }
            3 => Self::Reach {
                reach_m: r.f32()?,
                dropped: r.regions()?,
            },
            4 => Self::Got {
                regions: r.regions()?,
            },
            5 => Self::Resync { region: r.u32()? },
            6 => Self::Presence { pose: r.pose()? },
            7 => {
                let (id, with) = (r.u32()?, r.u32()?);
                let transaction = match r.u8()? {
                    1 => Transaction::Give {
                        slot: r.u8()?,
                        count: r.u16()?,
                    },
                    2 => Transaction::Mark { cell: r.u32()? },
                    other => return Err(Decode::Unknown(other)),
                };
                Self::Transaction {
                    id,
                    with,
                    transaction,
                }
            }
            8 => Self::Chat { text: r.text()? },
            9 => Self::Look { look: r.look()? },
            other => return Err(Decode::Unknown(other)),
        };
        r.end(message)
    }
}

impl ToClient {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = W(Vec::new());
        match self {
            Self::Welcome {
                player,
                clock_s,
                seed,
                generator,
                topology,
            } => {
                w.u8(1);
                w.u32(*player);
                w.f64(*clock_s);
                w.u64(*seed);
                w.u32(*generator);
                w.u32(*topology);
            }
            Self::Rejected { reason } => {
                w.u8(2);
                w.u8(code_of(&JOIN_REFUSALS, reason));
            }
            Self::Accepted { id, region, seq } => {
                w.u8(3);
                w.u32(*id);
                w.u32(*region);
                w.u64(*seq);
            }
            Self::Refused { id, reason } => {
                w.u8(4);
                w.u32(*id);
                w.u8(code_of(&REFUSALS, reason));
            }
            Self::Change {
                region,
                seq,
                change,
            } => {
                w.u8(5);
                w.u32(*region);
                w.u64(*seq);
                match *change {
                    Change::Edit {
                        cell,
                        layer,
                        material,
                    } => {
                        w.u8(1);
                        w.u32(cell);
                        w.u16(layer);
                        w.u8(material_code(material));
                    }
                }
            }
            Self::Snapshot { region, seq, bytes } => {
                w.u8(6);
                w.u32(*region);
                w.u64(*seq);
                w.bytes(bytes);
            }
            Self::Digest {
                region,
                seq,
                digest,
            } => {
                w.u8(7);
                w.u32(*region);
                w.u64(*seq);
                w.u64(*digest);
            }
            Self::Presence { player, pose } => {
                w.u8(8);
                w.u32(*player);
                w.pose(pose);
            }
            Self::Clock { clock_s } => {
                w.u8(9);
                w.f64(*clock_s);
            }
            Self::Settled { id, done } => {
                w.u8(10);
                w.u32(*id);
                w.u8(u8::from(*done));
            }
            Self::Chat { from, text } => {
                w.u8(11);
                w.u32(*from);
                w.bytes(text.as_bytes());
            }
            Self::Slots { slots } => {
                w.u8(12);
                for stack in slots.iter() {
                    w.stack(stack);
                }
            }
            Self::Drop { drop } => {
                w.u8(13);
                w.u64(drop.id);
                w.stack(Some(crate::inventory::Stack {
                    item: drop.item,
                    count: drop.count,
                }));
                w.f64(drop.position.x);
                w.f64(drop.position.y);
                w.f64(drop.position.z);
                w.f64(drop.made_s);
            }
            Self::DropLeft { id, left } => {
                w.u8(14);
                w.u64(*id);
                w.u16(*left);
            }
            Self::Gone { player } => {
                w.u8(15);
                w.u32(*player);
            }
            Self::Look { player, look } => {
                w.u8(17);
                w.u32(*player);
                crate::townsfolk::character::encode(look, &mut w.0);
            }
            Self::Record { record } => {
                w.u8(16);
                w.bytes(record.kind.as_bytes());
                w.u64(record.id);
                w.u32(record.schema);
                w.bytes(record.body.as_bytes());
            }
        }
        w.0
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Decode> {
        let mut r = R(bytes);
        let message = match r.u8()? {
            1 => Self::Welcome {
                player: r.u32()?,
                clock_s: r.f64()?,
                seed: r.u64()?,
                generator: r.u32()?,
                topology: r.u32()?,
            },
            2 => Self::Rejected {
                reason: of_code(&JOIN_REFUSALS, r.u8()?)?,
            },
            3 => Self::Accepted {
                id: r.u32()?,
                region: r.u32()?,
                seq: r.u64()?,
            },
            4 => Self::Refused {
                id: r.u32()?,
                reason: of_code(&REFUSALS, r.u8()?)?,
            },
            5 => {
                let (region, seq) = (r.u32()?, r.u64()?);
                let change = match r.u8()? {
                    1 => Change::Edit {
                        cell: r.u32()?,
                        layer: r.u16()?,
                        material: r.material()?,
                    },
                    other => return Err(Decode::Unknown(other)),
                };
                Self::Change {
                    region,
                    seq,
                    change,
                }
            }
            6 => Self::Snapshot {
                region: r.u32()?,
                seq: r.u64()?,
                bytes: r.bytes(MAX_SNAPSHOT)?,
            },
            7 => Self::Digest {
                region: r.u32()?,
                seq: r.u64()?,
                digest: r.u64()?,
            },
            8 => Self::Presence {
                player: r.u32()?,
                pose: r.pose()?,
            },
            9 => Self::Clock { clock_s: r.f64()? },
            10 => {
                let id = r.u32()?;
                let done = match r.u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err(Decode::Invalid),
                };
                Self::Settled { id, done }
            }
            11 => Self::Chat {
                from: r.u32()?,
                text: r.text()?,
            },
            12 => {
                let mut slots = crate::inventory::Slots::new();
                for index in 0..crate::inventory::CARRIED {
                    slots.set(index, r.stack()?);
                }
                Self::Slots { slots }
            }
            13 => {
                let id = r.u64()?;
                // A drop holds something: an empty one is never sent.
                let stack = r.stack()?.ok_or(Decode::Invalid)?;
                let position = DVec3::new(r.f64()?, r.f64()?, r.f64()?);
                Self::Drop {
                    drop: crate::drops::ItemDrop {
                        id,
                        item: stack.item,
                        count: stack.count,
                        position,
                        made_s: r.f64()?,
                    },
                }
            }
            14 => Self::DropLeft {
                id: r.u64()?,
                left: r.u16()?,
            },
            15 => Self::Gone { player: r.u32()? },
            17 => Self::Look {
                player: r.u32()?,
                look: r.look()?,
            },
            16 => {
                let kind = r.text()?;
                let id = r.u64()?;
                let schema = r.u32()?;
                let body = String::from_utf8(r.bytes(MAX_RECORD)?).map_err(|_| Decode::Invalid)?;
                Self::Record {
                    record: crate::records::Record {
                        kind,
                        id,
                        schema,
                        body,
                    },
                }
            }
            other => return Err(Decode::Unknown(other)),
        };
        r.end(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn versions() -> Versions {
        Versions {
            protocol: PROTOCOL,
            protocol_min: PROTOCOL_MIN,
            topology: 1,
            generator: 6,
            content: 0x1234_5678_9abc_def0,
            world: 42,
        }
    }

    fn pose() -> Pose {
        Pose {
            at: WorldPos::new(FrameId(3), DVec3::new(1.5, -2.25e6, 3.0e-3)),
            yaw: 1.25,
            pitch: -0.5,
        }
    }

    fn every_to_server() -> Vec<ToServer> {
        vec![
            ToServer::Hello {
                versions: versions(),
                key: [7; 16],
                name: "Ada".into(),
            },
            ToServer::Request {
                id: 9,
                request: Request::Dig {
                    cell: 0x1234_5678,
                    layer: 200,
                },
            },
            ToServer::Request {
                id: 10,
                request: Request::Place {
                    cell: 5,
                    layer: 201,
                    slot: 3,
                },
            },
            ToServer::Request {
                id: 11,
                request: Request::Pick { drop: 9 },
            },
            ToServer::Request {
                id: 13,
                request: Request::Plant {
                    cell: 9,
                    layer: 7,
                    slot: 2,
                },
            },
            ToServer::Request {
                id: 14,
                request: Request::Harvest { cell: 9 },
            },
            ToServer::Request {
                id: 12,
                request: Request::Toss {
                    slot: 39,
                    count: 7,
                    at_mm: [-4_800_123, 17, 2_000_000_000_000],
                },
            },
            ToServer::Reach {
                reach_m: 300.0,
                dropped: vec![1, 2, 3],
            },
            ToServer::Got { regions: vec![] },
            ToServer::Resync { region: 77 },
            ToServer::Presence { pose: pose() },
            ToServer::Transaction {
                id: 1,
                with: 2,
                transaction: Transaction::Give { slot: 4, count: 12 },
            },
            ToServer::Transaction {
                id: 2,
                with: 0,
                transaction: Transaction::Mark { cell: 99 },
            },
            ToServer::Chat {
                text: "hello, ünïcode".into(),
            },
            ToServer::Look {
                look: crate::townsfolk::character::rolled(&[3; 16]),
            },
        ]
    }

    fn every_to_client() -> Vec<ToClient> {
        let mut out = vec![
            ToClient::Welcome {
                player: 1,
                clock_s: 12_345.5,
                seed: 0xfeed_beef,
                generator: 15,
                topology: 1,
            },
            ToClient::Accepted {
                id: 9,
                region: 3,
                seq: u64::MAX,
            },
            ToClient::Change {
                region: 3,
                seq: 10,
                change: Change::Edit {
                    cell: 8,
                    layer: 150,
                    material: Material::Torch,
                },
            },
            ToClient::Change {
                region: 3,
                seq: 11,
                change: Change::Edit {
                    cell: 8,
                    layer: 151,
                    material: Material::flowing(3, true),
                },
            },
            ToClient::Snapshot {
                region: 4,
                seq: 2,
                bytes: vec![1, 2, 3, 4, 5],
            },
            ToClient::Digest {
                region: 4,
                seq: 2,
                digest: 0xdead_beef,
            },
            ToClient::Presence {
                player: 2,
                pose: pose(),
            },
            ToClient::Clock { clock_s: 0.0 },
            ToClient::Settled { id: 5, done: true },
            ToClient::Chat {
                from: 0,
                text: String::new(),
            },
            ToClient::Slots {
                slots: one_of_each(),
            },
            ToClient::Slots {
                slots: crate::inventory::Slots::new(),
            },
            ToClient::Drop {
                drop: crate::drops::ItemDrop {
                    id: 41,
                    item: crate::inventory::Item::Block(Material::Dirt),
                    count: 1,
                    position: DVec3::new(1.0e6, -2.5, 3.0e-4),
                    made_s: 812.25,
                },
            },
            ToClient::DropLeft { id: 41, left: 0 },
            ToClient::Gone { player: 3 },
            ToClient::Look {
                player: 2,
                look: crate::townsfolk::character::rolled(&[9; 16]),
            },
            ToClient::Record {
                record: crate::records::Record {
                    kind: "plot".into(),
                    id: 123_456,
                    schema: 1,
                    body: "(layer:7,crop:Some(\"wheat\"),planted_s:10.0)".into(),
                },
            },
        ];
        out.extend(
            JOIN_REFUSALS
                .iter()
                .map(|&reason| ToClient::Rejected { reason }),
        );
        out.extend(
            REFUSALS
                .iter()
                .map(|&reason| ToClient::Refused { id: 1, reason }),
        );
        out
    }

    /// Design 24: a forward made into yaw and back is the same tangent,
    /// anywhere on the sphere, the poles of Z and Y included.
    #[test]
    fn yaw_and_forward_round_trip_everywhere() {
        let ups = [
            DVec3::X,
            DVec3::Y,
            DVec3::Z,
            -DVec3::Z,
            DVec3::new(0.3, -0.2, 0.93).normalize(),
            DVec3::new(-0.7, 0.7, 0.1).normalize(),
        ];
        for up in ups {
            let (east, north) = tangent_frame(up);
            assert!(east.dot(up).abs() < 1e-12 && north.dot(up).abs() < 1e-12);
            for k in 0..12 {
                let yaw = -3.0 + 0.5 * k as f32;
                let f = forward_of(yaw, up);
                assert!((f.length() - 1.0).abs() < 1e-9 && f.dot(up).abs() < 1e-9);
                let back = forward_of(yaw_of(f, up), up);
                assert!((back - f).length() < 1e-6, "{up:?} {yaw}");
            }
        }
    }

    /// Carried slots with one of each kind of item, at both ends of the
    /// array (design 22).
    fn one_of_each() -> crate::inventory::Slots {
        use crate::inventory::{CARRIED, Item, Slots, Stack, Tool};
        let mut slots = Slots::new();
        slots.set(0, Some(Stack::new(Item::Block(Material::Stone), 12)));
        slots.set(3, Some(Stack::new(Item::Tool(Tool::Axe), 1)));
        slots.set(9, Some(Stack::new(Item::Fish(7), 2)));
        slots.set(
            CARRIED - 1,
            Some(Stack::new(Item::Good(crate::goods::Good::IRON_INGOT), 5)),
        );
        slots
    }

    /// Design 22: a good goes by its code, and a code this build lacks, or
    /// a count past the stack's limit, fails to decode.
    #[test]
    fn a_slot_this_build_cannot_hold_fails_to_decode() {
        let good = crate::goods::Good::IRON_INGOT;
        let bytes = ToClient::Slots {
            slots: one_of_each(),
        }
        .encode();
        let code = good.code().as_bytes();
        let at = bytes
            .windows(code.len())
            .position(|w| w == code)
            .expect("the good goes by its code");
        let mut unknown = bytes.clone();
        unknown[at] = b'#';
        assert_eq!(ToClient::decode(&unknown), Err(Decode::Invalid));
        let mut too_many = bytes;
        // The first slot's count: past any stack's limit.
        too_many[1..3].copy_from_slice(&u16::MAX.to_le_bytes());
        assert_eq!(ToClient::decode(&too_many), Err(Decode::Invalid));
    }

    /// Task 3.2: every message round-trips, and every shorter prefix of it
    /// is refused rather than read as something else.
    #[test]
    fn every_message_round_trips_and_a_cut_one_is_refused() {
        for m in every_to_server() {
            let bytes = m.encode();
            assert_eq!(ToServer::decode(&bytes), Ok(m.clone()));
            for n in 0..bytes.len() {
                assert!(ToServer::decode(&bytes[..n]).is_err(), "{m:?} cut at {n}");
            }
            let mut longer = bytes.clone();
            longer.push(0);
            assert_eq!(ToServer::decode(&longer), Err(Decode::Trailing));
        }
        for m in every_to_client() {
            let bytes = m.encode();
            assert_eq!(ToClient::decode(&bytes), Ok(m.clone()));
            for n in 0..bytes.len() {
                assert!(ToClient::decode(&bytes[..n]).is_err(), "{m:?} cut at {n}");
            }
        }
    }

    /// Task 3.2: a message from a newer build, with a tag this one does not
    /// know, is refused cleanly as unknown; so is an unknown request or
    /// change inside a known message.
    #[test]
    fn an_unknown_newer_message_is_refused_cleanly() {
        assert_eq!(ToServer::decode(&[200, 1, 2, 3]), Err(Decode::Unknown(200)));
        assert_eq!(ToClient::decode(&[99]), Err(Decode::Unknown(99)));
        let mut request = ToServer::Request {
            id: 1,
            request: Request::Dig { cell: 1, layer: 1 },
        }
        .encode();
        request[5] = 40;
        assert_eq!(ToServer::decode(&request), Err(Decode::Unknown(40)));
        let mut change = ToClient::Change {
            region: 1,
            seq: 1,
            change: Change::Edit {
                cell: 1,
                layer: 1,
                material: Material::Air,
            },
        }
        .encode();
        change[13] = 41;
        assert_eq!(ToClient::decode(&change), Err(Decode::Unknown(41)));
    }

    /// A value no message has is refused: a material code nothing names, a
    /// non-finite position, a list longer than its bound, bad UTF-8.
    #[test]
    fn a_value_no_message_has_is_refused() {
        let mut change = ToClient::Change {
            region: 1,
            seq: 1,
            change: Change::Edit {
                cell: 1,
                layer: 1,
                material: Material::Air,
            },
        }
        .encode();
        *change.last_mut().unwrap() = 250;
        assert_eq!(ToClient::decode(&change), Err(Decode::Invalid));
        let mut presence = ToServer::Presence { pose: pose() }.encode();
        presence[3..11].copy_from_slice(&f64::NAN.to_le_bytes());
        assert_eq!(ToServer::decode(&presence), Err(Decode::Invalid));
        let mut got = vec![4];
        got.extend_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(ToServer::decode(&got), Err(Decode::Invalid));
        let mut chat = vec![8];
        chat.extend_from_slice(&2u32.to_le_bytes());
        chat.extend_from_slice(&[0xff, 0xfe]);
        assert_eq!(ToServer::decode(&chat), Err(Decode::Invalid));
    }

    /// Design 9 and 15.9: the protocol is a range; the world's versions and
    /// identity are exact.
    #[test]
    fn a_join_agrees_on_a_protocol_range_and_the_exact_world() {
        let server = Versions {
            protocol: 3,
            protocol_min: 2,
            ..versions()
        };
        let older = Versions {
            protocol: 2,
            protocol_min: 1,
            ..versions()
        };
        assert_eq!(agree(&server, &older), Ok(2));
        let ancient = Versions {
            protocol: 1,
            protocol_min: 1,
            ..versions()
        };
        assert_eq!(agree(&server, &ancient), Err(JoinRefusal::GameVersion));
        let other_generator = Versions {
            generator: 5,
            ..server
        };
        assert_eq!(
            agree(&server, &other_generator),
            Err(JoinRefusal::WorldVersion)
        );
        let other_world = Versions {
            world: 43,
            ..server
        };
        assert_eq!(
            agree(&server, &other_world),
            Err(JoinRefusal::DifferentWorld)
        );
        assert_eq!(agree(&versions(), &versions()), Ok(PROTOCOL));
        let anywhere = Versions {
            world: 0,
            generator: 0,
            ..server
        };
        assert_eq!(
            agree(&server, &anywhere),
            Ok(3),
            "world 0 joins any world, any generator"
        );
        let other_topology = Versions {
            topology: 9,
            ..anywhere
        };
        assert_eq!(
            agree(&server, &other_topology),
            Err(JoinRefusal::WorldVersion)
        );
    }
}
