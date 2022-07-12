# Taipo

Taipo is an experimental game exploring the idea of limiting control of the game to typing random Japanese phrases.

This could turn out to be a useful tool to practice quick Japanese recognition / production.

This is my first foray into ECS and it is a bit of a mess.

## Demo

It is entirely possible that there is a demo available on [itch.io](https://euclidean-whale.itch.io/taipo).

## Build

Taipo uses the [bevy 0.7](https://bevyengine.org/) engine and is pretty easy to build.

### Build Dependencies

- [rust 1.57](https://www.rust-lang.org/tools/install)

Bevy also has a few [dependencies](https://bevyengine.org/learn/book/getting-started/setup/) you may need.

### Build Taipo

```bash
cargo run --profile release
```

### For web

```bash
cargo install wasm-server-runner
cargo run --profile release --target=wasm32-unknown-unknown
```

## TODO

- [ ] Corpses should despawn after some time. (This might break the gameover screen currently)
- [ ] You should be able to type "tsuduku" on the game over screen to restart
- [ ] Load tower stats from external game data. (game.ron or with map data)
- [ ] Load starting yen from map data
- [ ] Position tower label placeholders in editor with a direction attribute (up/down/left/right)
- [ ] Add a "partially typed" state to rendered glyphs?
- [ ] If you "overtype" a word, it should be highlighted differently
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
  - [ ] Decorations
  - [ ] Shuriken Tower is awful, so maybe that too

## Attribution

We're temporarily using some unmodified assets from [BrowserQuest](https://github.com/mozilla/BrowserQuest) which are licensed under CC-BY-SA 3.0.
