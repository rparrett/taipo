# Taipo

Taipo is an experimental game exploring the idea of limiting control of the game to typing random Japanese phrases.

This could turn out to be a useful tool to practice quick Japanese recognition / production.

Currently targeting wasm/webgl only, but there's no reason we couldn't do native desktop builds too.

This is my first foray into ECS and it is a bit of a mess.

## Demo

It is entirely possible that there is a demo running here: [https://taipo.robparrett.com](https://taipo.robparrett.com)

## Build

Taipo uses the [bevy](https://bevyengine.org/) engine (currently tracking the master branch pre-0.5) and is pretty easy to build.

### Build Dependencies

- [rust stable](https://www.rust-lang.org/tools/install)
- [cargo-make](https://github.com/sagiegurari/cargo-make#installation)

Bevy also has a few [dependencies](https://bevyengine.org/learn/book/getting-started/setup/) on linux and windows.

### Build

```
git clone git@github.com:rparrett/taipo.git && cd taipo
cargo make serve --profile=release
```

## TODO

- [ ] Corpses should despawn after some time. (This might break the gameover screen currently)
- [ ] You should be able to type "tsuduku" on the game over screen to restart
- [ ] Load tower stats from external game data. (game.ron or Tiled?)
- [ ] Improve word list parsing so that parenthesized "rendered text" is optional for ascii, hiragana and katakana
- [ ] Add a "partially typed" state to rendered glyphs?
- [ ] If you "overtype" a word, it should be highlighted differently
- [x] Workaround lack of sound in bevy on the web. Add sound for
  - [x] Mis-typed character
  - [ ] Mis-entered action
  - [ ] Correctly entered action
  - [ ] Tower firing
  - [ ] Enemy taking damage
  - [ ] Enemy dealing damage
  - [ ] Becoming able to afford to do literally anything
- [ ] Position tower label placeholders in editor? Maybe just with a direction attribute?
- [ ] Commission some art, get rid of BrowserQuest assets
  - [ ] Enemies
  - [ ] Shuriken Tower
  - [ ] Goal (Small Rural Train Station?)

## Attribution

We're temporarily using some unmodified assets from [BrowserQuest](https://github.com/mozilla/BrowserQuest) which are licensed under CC-BY-SA 3.0.
