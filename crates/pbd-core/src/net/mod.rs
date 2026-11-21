//! What a shared world agrees on (`openspec/changes/multiplayer`): the
//! region's canonical state and digest, and the protocol's messages. Pure
//! functions and plain data, with no socket, thread or engine: the server,
//! the client and the tests all call the same code.

pub mod digest;
