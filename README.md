# md-500-access-rs

SysEx codec, CLI, and WASM bindings for the **BOSS MD-500** dual modulation
pedal — one of the device "access" crates that feed the
[`midi-ccess`](https://github.com/ragb/midi-ccess) accessible editor, built on
[`midi-access-kit`](https://github.com/ragb/midi-access-kit).

Implemented from the official **MD-500 MIDI Implementation v1.00** (2017-07-01)
and **verified on real hardware** (2026-06-12: identity, all-area dump/decode,
and chunked write-with-verify) — see [`docs/sysex-notes.md`](docs/sysex-notes.md).

## Crates

| Crate | What it is |
|-------|-----------|
| **`md500-core`** | Pure codec (`Device` impl, typed area models, SysEx framing). No I/O; compiles for `wasm32-unknown-unknown`. |
| **`md500`** | CLI — the generic `midi-access-cli` engine dispatched through `Device for Md500`. |
| **`md500-wasm`** | `wasm-bindgen` + TypeScript bindings for the editor. |

## Areas

| Area | Address | Contents |
|------|---------|----------|
| `setup` | `00 00 00 00` | currently-selected patch |
| `system` | `10 00 00 00` (+ `10 00 10 00`) | global System Common + System Control |
| `pc-map` | `20 00 00 00` | 384-entry MIDI Program Change → patch map |
| `bank` | `30 00 00 00` | Temporary bank SIMUL parameters |
| `patch-a` / `patch-b` / `patch-c` | `30 00 10/20/30 00` | the Temporary bank's three 586-byte modulation patches |

A patch covers all twelve modulation modes (Chorus, Flanger, Phaser, C-Vibe,
Vibrato, Tremolo, Dimension, Ring Mod, Rotary, Filter, Slicer, Overtone), the
shared LFO/EQ/output header, the per-patch footswitch/expression control, and
eight ASSIGN matrix slots. The 99 stored user banks are addressable from the
wasm layer (`bankBase(n)` / `patchBase(n, "a")`).

## CLI

```
md500 dump <area> -o <file>          # read an area off the device into YAML
md500 sync <area> -i <file> [--verify]
md500 show <file>                    # identify + pretty-print (offline)
md500 lint <file>                    # validate through the typed model (offline)
md500 diff <a> <b>                   # field-by-field diff (offline)
md500 schema <area>                  # JSON Schema for an area (offline)
md500 catalog                        # params + catalogs + defaults as JSON (offline)
md500 resolve <file>                 # value names → numbers, codec-ready YAML (offline)
md500 identity                       # Universal Identity Request probe
md500 ports                          # list MIDI ports
```

Global options: `--port` / `--input-port` / `--output-port`, `--device <0..15>`,
`-v/--verbose`, `-q/--quiet`. `--device` maps directly to the MD-500 wire device
id (`0x00..`).

## Names and metadata

`md500 catalog` (and the wasm `catalog()` / `paramCatalog()` / `fieldHelp()` /
`fieldMeta()`) expose a label / group / help entry for every field plus two
name↔number tables — `assign_source` (70) and `assign_target` (265) — so a preset
can be authored by name and the editor can render the right control. `md500
resolve <file>` rewrites value names to numbers for the codec.

## Wire protocol

Roland SysEx, 4-byte model id `00 00 00 43`, RQ1 `11` / DT1 `12`, standard Roland
checksum over address+data. Addresses and RQ1 sizes are four 7-bit bytes.

Two behaviours confirmed on hardware and handled by the crate: requests are
addressed to the **broadcast device id `0x7F`** (the device answers regardless of
its configured id, which defaults to `0x10` — outside the `--device` range), and
large regions transfer as **multiple DT1 chunks** (≤242 bytes) at successive
addresses — reassembled on read, split on write. Full details and ready-to-send
test frames: [`docs/sysex-notes.md`](docs/sysex-notes.md).

## Development

```sh
cargo test --workspace --all-features
cargo fmt --all
cargo clippy --workspace --all-features --all-targets
cargo build -p md500-core --target wasm32-unknown-unknown --features tsify
# regenerate committed JSON Schemas:
for a in setup system pc-map bank; do cargo run -p md500 -- schema "$a" > schemas/md500-$a.schema.json; done
cargo run -p md500 -- schema patch-a > schemas/md500-patch.schema.json
```

## License

MIT.
