# Taipo

Taipo is an experimental game exploring the idea of limiting control of the game to typing random Japanese phrases.

This could turn out to be a useful tool to practice quick Japanese recognition / production.

Currently targeting wasm/webgl only, but there's no reason we couldn't do native desktop builds too.

This is my first foray into ECS and it is a bit of a mess.

## Demo

It is entirely possible that there is a demo available on [itch.io](https://euclidean-whale.itch.io/taipo) or [https://taipo.robparrett.com](https://taipo.robparrett.com).

## Build

Taipo uses the [bevy 0.5](https://bevyengine.org/) engine and is pretty easy to build.

### Build Dependencies

- [rust 1.51](https://www.rust-lang.org/tools/install)
- [cargo-make](https://github.com/sagiegurari/cargo-make#installation)

Bevy also has a few [dependencies](https://bevyengine.org/learn/book/getting-started/setup/) you may need.

### Build

```rs
git clone git@github.com:rparrett/taipo.git && cd taipo
cargo make serve --profile=release
cargo make run --profile=release
```

## TODO

- [ ] Corpses should despawn after some time. (This might break the gameover screen currently)
- [ ] You should be able to type "tsuduku" on the game over screen to restart
- [ ] Switch to `bevy_ecs_tilemap`
- [ ] Load tower stats from external game data. (game.ron or with map data)
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
