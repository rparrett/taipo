# Taipo

Taipo is an experimental game exploring the idea of limiting control of the game to typing random Japanese phrases. There's also an English mode.

This could turn out to be a useful tool to practice quick Japanese recognition / production.

## Play Online

A web build is hosted on [itch.io](https://euclidean-whale.itch.io/taipo).

## Contributing

Please feel free to open a PR if you are motivated. See the TODO list below and any open GitHub issues.

## Build

Taipo uses the [Bevy](https://bevyengine.org/) engine and is pretty easy to build.

### Build Dependencies

- [Rust](https://www.rust-lang.org/tools/install)

Bevy also has a few [dependencies](https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies) on Windows and Linux that you may need.

### Build Taipo

```bash
cargo run --release
```

### For the web

```bash
cargo install --git https://github.com/TheBevyFlock/bevy_cli --locked bevy_cli --rev 326722d --features wasm-opt
bevy run --release web
```

## TODO

- [ ] Move UI images into texture atlas.
- [ ] Corpses should despawn after some time.
- [ ] Load tower stats from `game.ron`.
- [ ] Load starting yen from `game.ron`.
- [ ] Position tower label placeholders in editor with a direction attribute (up/down/left/right)
- [ ] Add a "partially typed" state to rendered glyphs?
- [ ] If you type extra letters at the end of a prompt, but it otherwise matches, we should tint it red, not green.
- [ ] Display upcoming wave's enemy type
- [ ] Add sound for
  - [ ] Wave complete (Train Station Jingle?)
  - [ ] Wave approaching
  - [ ] Enemy dealing damage
  - [ ] ?Earning yen
  - [ ] ?Becoming able to afford to do literally anything
  - [ ] ?Tower firing
  - [ ] ?Enemy taking damage
  - [ ] ?Wrong word after pressing enter
- [ ] Art
  - [ ] Enemies (Last remaining BrowserQuest assets)
  - [ ] Map decorations
  - [ ] An auto-tilable tileset
  - [ ] Shuriken Tower is awful, so maybe that too
  - [ ] Another tower or two
- [ ] More levels!
- [ ] More words, word lists.
- [ ] Allow users to add their own words

## Attribution

We're temporarily using some unmodified assets from [BrowserQuest](https://github.com/mozilla/BrowserQuest), which are licensed under CC-BY-SA 3.0.
