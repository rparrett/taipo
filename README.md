# Taipo

Taipo is an experimental game exploring the idea of limiting control of the game to typing random Japanese phrases. There's also an English mode.

This could turn out to be a useful tool to practice quick Japanese recognition / production.

This is my first foray into ECS, and it is a bit of a mess.

## Play Online

A web build is hosted on [itch.io](https://euclidean-whale.itch.io/taipo).

## Build

Taipo uses the [Bevy 0.8](https://bevyengine.org/) engine and is pretty easy to build.

### Build Dependencies

- [Rust](https://www.rust-lang.org/tools/install)

Bevy also has a few [dependencies](https://bevyengine.org/learn/book/getting-started/setup/) on Windows and Linux that you may need.

### Build Taipo

```bash
cargo run --release
```

### For web

```bash
cargo install cargo-make
cargo make --profile release serve
```

## TODO

- [ ] Replace main menu with a typing interface
- [ ] Can we remove `TaipoStage::AfterUpdate`?
- [ ] Corpses should despawn after some time. (This might break the gameover screen)
- [ ] Load tower stats from `game.ron`.
- [ ] Load starting yen from `game.ron`.
- [ ] Position tower label placeholders in editor with a direction attribute (up/down/left/right)
- [ ] Add a "partially typed" state to rendered glyphs?
- [ ] If you type extra letters at the end of a target, but it otherwise matches, we should tint it red, not green.
- [ ] Pressing escape should clear the typing buffer.
- [ ] Display upcoming wave's enemy type
- [ ] Add some volume control, even if it's just typing "quieter" and "louder"
- [ ] Add sound for
  - [ ] Wrong word after pressing enter
  - [ ] Correct word after pressing enter
  - [ ] Wave complete (Train Station Jingle?)
  - [ ] Becoming able to afford to do literally anything
  - [ ] Enemy dealing damage
  - [ ] ?Tower firing
  - [ ] ?Enemy taking damage
- [ ] Commission some art
  - [ ] Enemies (Last remaining BrowserQuest assets)
  - [ ] Map decorations
  - [ ] Shuriken Tower is awful, so maybe that too
- [ ] Refactor so that we can restart the game without exiting and reopening.
- [ ] More levels!

## Attribution

We're temporarily using some unmodified assets from [BrowserQuest](https://github.com/mozilla/BrowserQuest), which are licensed under CC-BY-SA 3.0.
