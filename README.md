xrw
====
Query structured binary formats and perform modifications.

Most structured binary formats share a common design: an identifier, a size, and a payload. This library describes each format family with a [`Descriptor`] and provides a dedicated parser for each.

Parsing produces a [`Layout`]: an ordered index of [`Block`]s, each holding its own identifier, position, and payload size. Payloads are not stored or read during parsing. The layout can be queried, manipulated, and written back to a new file without ever interpreting what the blocks contain.

Currently supported families include IFF/RIFF/RF64/BW64/Sony Wave64, CAF, and ISOBMFF. Support for PNG, JPEG, EBML/Matroska, FLAC, Ogg, TIFF/IFD, ASF, MXF, and more is planned.

See the [documentation](https://docs.rs/xrw) for the complete reference.

### Related
[umedia](https://github.com/ubmu/umedia) - Extensive multimedia metadata and extradata parsing and editing.

This library provides in-depth metadata parsing and manipulation for multimedia files. This will not solely be a tagging library. The aim is for any known piece of data for a given file format can be parsed and written.

[streaminfo](https://github.com/ubmu/streaminfo) - Extract codec parameters and stream information from multimedia files.

This library is similar to [`ffprobe`].

[adhere](https://github.com/ubmu/adhere) - Specification compliance checking for multimedia file formats.

[`xrw`] only ensures that the container is written correctly, and does not care if the file format itself is valid (e.g. block ordering, required blocks, specific number of certain blocks...). [`adhere`] takes in a [`Layout`], performs file format-specific checks, and repairs the [`Layout`] if invalid. In your workflow
you can pass it in before applying changes with `xrw`. For example:

```rust
let mut layout = Layout::open("invalid_order.wav")?;
let audit = adhere::audit(&mut layout)?;
if audit.has_errors() {
    adhere::repair(&mut layout)?;
}
layout.save("valid_order.wav")?;
```
Or:
```rust
let mut layout = Layout::open("invalid_order.wav")?;
adhere::validate(&mut layout)?;
layout.save("valid_order.wav")?;
```


TODO: Update to handle nested structures. Add test cases. Implement write for Inter, Core, Base.
After all of this, begin implementing `adhere` for these formats. Then as more formats are supported,
we implement the adhere validation as we go.
