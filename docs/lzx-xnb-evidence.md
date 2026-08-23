# Managed XNA LZX XNB evidence

Status date: 2026-08-23

## Scope and authority

This implementation handles the compression framing used by XNA 4.0 Windows
XNB v5. XNA's `ContentReader.PrepareStream` metadata/IL is authoritative: a
compressed file has the ordinary 10-byte XNB header, a 32-bit little-endian
decompressed payload size, and a compressed byte count equal to the declared
file size minus 14. Decompression replaces only the payload before the normal
managed reader table and object-graph pipeline runs.

The implementation is not a generic LZX or MonoGame-container promise. LZ4,
DXT texture data, effect bytecode, and other compression concepts remain
separate. Uncompressed XNB framing follows its existing path unchanged.

## Framing and decoder

After the 14-byte compressed header, a normal frame starts with a two-byte
big-endian compressed block length. A first byte of `0xff` selects the
five-byte extended header: a two-byte decompressed frame length followed by a
two-byte compressed block length. A short-header frame decompresses to 32 KiB;
an extended frame may be smaller, but never larger.

One stateful decoder with a 64 KiB window is retained for the entire asset.
The decoder implements LZX verbatim, aligned, and uncompressed blocks,
pretree/main/length/aligned Huffman tables, repeated offsets, window wrap, and
bitstream bounds. XNB does not use the optional CAB Intel E8 transform, so a
stream advertising it is rejected.

The frame layer requires exact per-frame output and exact equality with the
declared decompressed payload size. It accepts an exact physical end or the
canonical zero end marker/padding. Truncated headers/blocks and non-zero data
after the marker fail before reader publication.

## Deterministic evidence

Repository-owned synthetic fixtures cover:

- single extended frame and multi-frame decoding with persistent state;
- one legal 32 KiB short-header frame;
- compressed primitive loading through the ordinary reader table;
- shared-resource fixups and identity;
- compressed external references, including a compressed child asset;
- reader failure after decompression and partial-resource rollback;
- cache identity, `Unload`, and reload identity replacement;
- a complete compressed Model graph with shared buffers/effect, draw, unload,
  retained-child invalidation, and ten native ownership cycles.

There are 14 focused negative framing/container cases: oversized declared
decompressed size, truncated short header, truncated extended header,
truncated compressed block, zero compressed block size, zero frame size,
oversized frame, output longer than declared, output shorter than declared,
decoder failure, truncated trailing header, non-zero trailing marker data,
truncated compressed XNB payload header, and mismatched declared file size.
Canonical zero end padding is also accepted explicitly.

As an independent qualification of the compressed Huffman paths, the optional
test selected by `CNA_RUST_LZX_FIXTURE_DIR` compared two read-only legal
MonoGame-produced XNB files byte-for-byte with independently decompressed
outputs of 16,561 and 44,032 bytes. The latter spans multiple 32 KiB frames.
Those external assets are not copied or committed here.

## Result boundary

LZX framing is managed-complete. The synthetic compressed Model uses the same
native constructors and draw route as the uncompressed Model reader, and the
qualified HEADLESS artifact completed all ten cycles without a crash,
double-free, or observed use-after-free. This establishes stream framing,
reader dispatch, and native resource ownership; it does not claim visible
rendered pixels or support for compressed texture surface formats.
