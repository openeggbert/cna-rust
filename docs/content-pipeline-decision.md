# The Content Pipeline product boundary (RUST-XNA-004)

Decision date: 2026-08-31. Measured against cnanext `599d14e5`, ABI 0.21, and
the SHA-256-admitted `Microsoft.Xna.Framework.Content.Pipeline` assemblies.

## The question

With the ten retained Microsoft **runtime** assemblies projected at strict zero,
the only structural gap in the whole retained corpus is the design-time Content
Pipeline: **125 types, 743 reference members**. The superset profile's
`missing = 125` is exactly and only this.

The question was never "can they be implemented". It is whether they belong in
this product at all. Implementing 125 types to turn a number into zero would be
the wrong reason to do anything.

## What the 125 actually are

Every one is classified, with no remainder:

| Count | Category | What it is |
|---:|---|---|
| 52 | design-time content | `BitmapContent`, `MeshContent`, `NodeContent`, the `MaterialContent` family, `VertexChannel`, `AudioContent`, `VideoContent` -- the authoring-time object model |
| 33 | processors | `ModelProcessor`, `TextureProcessor`, `FontDescriptionProcessor`, their compiled outputs, plus `ContentProcessor`, `IContentProcessor`, `ContentProcessorContext` and processor parameters |
| 13 | importers | `FbxImporter`, `XImporter`, `EffectImporter`, `TextureImporter`, the four media importers, `XmlImporter`, `FontDescriptionImporter`, plus `ContentImporter`, `IContentImporter`, `ContentImporterContext` |
| 10 | build plumbing | `ContentBuildLogger`, `ContentIdentity`, `ContentItem`, `ExternalReference`, `OpaqueDataDictionary`, `TargetPlatform`, the two exceptions |
| 5 | CLR attributes / reflection | `ContentImporterAttribute`, `ContentProcessorAttribute`, `ContentTypeWriterAttribute`, `ContentTypeSerializerAttribute`, `PipelineComponentScanner` |
| 5 | intermediate XML serializer | `IntermediateSerializer`, `IntermediateReader`, `IntermediateWriter`, `ContentTypeSerializer` and its `ChildCallback` |
| 4 | MSBuild tasks | `BuildContent`, `BuildXact`, `CleanContent`, `GetLastOutputs` |
| 3 | `.xnb` compiler/writer | `ContentCompiler`, `ContentTypeWriter`, `ContentWriter` |
| **125** | | |

## Three findings that decide it

### 1. Seventeen of them cannot be projected faithfully at all

Not "would be hard" -- cannot. The four MSBuild tasks derive from
`Microsoft.Build.Utilities.Task`: their entire contract is *being* an MSBuild
task, and a Rust crate cannot be one. The five attribute/reflection types exist
so that `PipelineComponentScanner` can walk a .NET assembly's metadata and
discover importers and processors by attribute; Rust has no attribute
reflection and no assembly to scan. The five intermediate-serializer types
round-trip arbitrary CLR object graphs to XML through `System.Type`. The three
`.xnb` writers exist to emit a format CNA does not use.

Projecting any of these would mean declaring a Rust type with the right name
whose behaviour is a stub. That is worse than absence: absence is honest.

### 2. CNA already has a native content pipeline, and this binding already uses it

This is the decisive one. CNA does not lack a design-time story that the XNA
pipeline would fill -- it has its own, entirely native, with no CLR, no MSBuild
and no reflection anywhere in it:

| XNA pipeline concept | CNA's own |
|---|---|
| importers (`TextureImporter`, `WavImporter`, ...) | `cna_cnb_import_image_as_texture2d`, `cna_cnb_import_dds_as_texture_cube`, `cna_cnb_import_wav_as_sound_effect` |
| processors + `ContentCompiler` producing `.xnb` | `cna_cnb_compile_cnj`: one `.cnj` document plus its binary sidecars in, one `.cnb` image out, for all eight asset types |
| `ExternalReference` and dependency tracking | the compile result's absorbed-file and external-reference lists, which upstream says are "what make it useful to a build system rather than only to a loader" |
| `ContentTypeWriter` + `ContentTypeWriterAttribute` discovery | `CnbWriter` plus the loader registry, where a game registers its own asset type by name |
| `Processors::ModelContent`, `SpriteFontContent`, `SoundEffectContent`, `VertexBufferContent` | the compiled `.cnb` Model, SpriteFont and SoundEffect this binding already reads and writes |

That last row is not a plan. `RUST-EXT-013` bound it this milestone: a Rust
consumer can author a model with bones, meshes, parts, geometry and materials,
encode it, parse it back, and get every value out exactly; the same for sprite
fonts and sound effects; and it can register a Rust loader for an asset type of
its own and get its own value back. The design-time need is already met, by the
tooling CNA actually ships.

### 3. The overlap that exists is compiled content, which is already bound

Of the 33 processor-family types, nine are the *compiled outputs* --
`ModelContent`, `ModelMeshContent`, `ModelBoneContent`, `ModelMeshPartContent`,
`VertexBufferContent`, `VertexDeclarationContent`, `SpriteFontContent`,
`SoundEffectContent`, `CompiledEffectContent` -- and every one has a `.cnb`
counterpart this binding reads and writes today. Adding the XNA-named versions
would give a Rust consumer two ways to say the same thing, one of which cannot
be produced by any tool CNA ships.

## The decision

**Option D: CNA's native `.cnj`/`.cnb` content tooling replaces direct
projection. The Content Pipeline is out of scope for this product.**

Not option A. Putting 125 design-time types in the runtime crate would triple
its public surface with types no game calls at run time, seventeen of which
would have to be stubs, purely to make a number zero. The brief's own rule
applies: do not pollute the runtime crate to achieve a numeric zero.

Not option B. A separate design-time crate would have to reimplement MSBuild
task integration and CLR attribute discovery to be faithful, and would then
duplicate a pipeline CNA already has natively. A second product needs a reason
better than symmetry with a 2010 toolchain.

Option C is folded in rather than chosen: the compatibility concepts that are
genuinely useful -- compiled model, sprite font, sound effect, external
references, a registry for a game's own asset types -- are already implemented,
under `cna::extensions::content`, in CNA's own shapes rather than XNA's names.

## What this means for the numbers

`PROFILE_SUPERSET_MISSING_TYPES = 125` is now a **stated product boundary**, not
a backlog. It should not be expected to fall, and a future session should not
treat it as work remaining. The two runtime profiles stay the hard gates:

```text
selected runtime profile   257 types, 2964 members, 0 missing, 0 diagnostics
full runtime profile       331 types, 3640 members, 0 missing, 0 diagnostics
pipeline profile           128 types,  743 members, 125 missing  -- out of scope
superset profile           459 types, 4383 members, 125 missing  -- the same 125
```

## What would reopen it

One thing: if CNA grew a *runtime* need for a design-time type -- if some route
began handing back something whose only faithful projection is, say,
`BitmapContent`. Nothing does today. A future session should re-derive this from
the live headers rather than assume the answer holds, and the classification
above is the shape to redo it in.
