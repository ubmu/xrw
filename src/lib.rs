/*!
A library for structural I/O across binary formats.

`xrw` treats any binary format whose data is organized into discrete, bounded units as a
container. These units, whether called chunks, atoms, boxes, elements, pages, or frames,
share a common shape: an identifying marker, a size, and a payload. The size may be stored
explicitly in the block header, fixed by the format specification, or derivable from header
fields. Regardless, `xrw` represents every unit as a [`Block`].

# Approach

Each container family is described by a [`Descriptor`], which stores byte order, identifier
width, size field width, and alignment boundaries. All reading and writing behaviour is derived
from this descriptor, making it straightforward to support new formats without changes to the
core traversal logic.

When given a stream, `xrw` reads the magic bytes to identify the container [`Family`] and,
where the format specifies it, its [`Kind`]. From there, the container header is parsed and
the stream is traversed block by block, recording each block's identifier, offset, payload
offset, and size into a [`Structure`]. Payloads are not read or copied.

The resulting [`Structure`] can be queried and manipulated freely. Reordering, removing, or
swapping blocks operates only on the in-memory block index. Regardless of how large the file is,
the cost of manipulation is negligible since no payload data is involved. Since every [`Block`]
retains its absolute offset into the original stream, the block index acts as a complete description
of what the output file should look like. To apply changes, [`Structure::write_from`] reads each
payload directly from its stored offset and writes it to a new file in the current block order,
handling size fields, padding, and alignment automatically.

Currently supported container families include RIFF, IFF, RF64, BW64, and Sony Wave64.
Support for QuickTime/ISOBMFF, PNG, JPEG, EBML/Matroska, FLAC, Ogg, TIFF/IFD, ASF, MXF,
and more is planned.

# Overview

[`Structure`] is the primary struct, representing the parsed result of a binary file. It
exposes the complete block index alongside the detected family, descriptor, and any
family-specific metadata such as [`DataSize64`] for RF64 and BW64 files.
```rust
let mut reader = Reader::open("audio.wav")?;
let structure = Structure::read(&mut reader, &ReadOptions::default())?;

if let Some(fmt) = structure.find(Marker::FourCC(*b"fmt ")) {
    let payload = structure.read_payload(&mut reader, fmt)?;
}
```

Supported operations include reading and writing containers, block manipulation, finding and
querying, padding and alignment, and container conversion. See [`Structure`] for the complete
list.
# Related

- [`quickparse`](https://github.com/) — minimal essential chunk extraction for audio formats.
- [`coreav`](https://github.com/) — comprehensive metadata reading and manipulation for audio and video formats.
- [`xver`](https://github.com/) — specification compliance checking for audio and video formats.
*/

#![warn(clippy::pedantic)]
// JUST FOR NOW UNTIL ALL FUNCTIONS ARE IMPLEMENTED
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod block;
pub mod descriptor;
pub mod family;
pub mod kind;
pub mod marker;
pub mod reader;
pub mod structure;

pub use block::{Block, BlockType};
pub use descriptor::Descriptor;
pub use family::Family;
pub use kind::Kind;
pub use marker::Marker;
pub use reader::Reader;
pub use structure::Structure;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Byteorder {
    Big,
    Little,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unknown container")]
    UnknownContainer,

    #[error("unexpected end of stream")]
    UnexpectedEof,
}

pub type Result<T> = std::result::Result<T, Error>;
