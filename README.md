# Taipo

Taipo is an experimental game exploring the idea of limiting control of the game to typing random Japanese phrases.

This could turn out to be a useful tool to practice quick Japanese recognition / production.

Currently targeting wasm/webgl only, but there's no reason we couldn't do native desktop builds too.

This is my first foray into ECS and it is a bit of a mess.

## Demo

It is entirely possible that there is a demo running here: [https://taipo.robparrett.com](https://taipo.robparrett.com)

## Build

Taipo uses the [bevy](https://bevyengine.org/) engine (currently tracking the master branch) and is pretty easy to build.

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

- [ ] Should we be running some systems on a fixed timestep?
- [ ] Corpses should despawn after some time.
- [ ] You should be able to type "tsuduku" on the game over screen to restart
- [ ] If you "overtype" a word, it should be highlighted differently
- [ ] Add pre-game buttons for selecting from different word-sets
- [ ] We should load game data externally in some serialized format
  - [X] Lexicon
  - [ ] Level
    - [X] Background Tiles
    - [X] Tower Slots
    - [X] Goal
    - [X] Enemy Path
    - [ ] Tower Stats
    - [ ] Enemy Waves
- [ ] Improve word parsing so hiragana/katakana are automatically converted to ascii (but can still be overridden with parenthesis)
- [ ] Add a "partially typed" state to rendered glyphs?
- [ ] When sound becomes possible in Bevy/web, things should make sounds
  - [ ] Mis-typed character
  - [ ] Mis-entered action
  - [ ] Correctly entered action
  - [ ] Tower firing
  - [ ] Enemy taking damage
  - [ ] Enemy dealing damage
  - [ ] Becoming able to afford to do literally anything
- [ ] Position tower label placeholders in editor? Maybe just with a direction attribute?
- [ ] Do an art?
  - [ ] Give up, bribe someone else to do an art
    - [ ] Train or Subway theme
- [ ] Deal with action ambiguity
  - [ ] Either prevent ambiguities when assigning words for targets
  - [ ] Or allow the player to tab through multiple completed targets
  - [X] Yen text should be red when action panel item is disabled because we can't afford it
  - [X] Action panel needs to get updated when money changes in case we can now afford something

## Attribution

We're temporarily using some assets from (BrowserQuest)[https://github.com/mozilla/BrowserQuest] which is licensed under CC-BY-SA 3.0.
