# Taipo

Taipo is an experimental game exploring the idea of limiting control of the game to typing random Japanese phrases.

This could turn out to be a useful tool to practice quick Japanese recognition / production.

Currently targeting wasm/webgl only, but there's no reason we couldn't do native desktop builds too.

This is my first foray into ECS and it is a bit of a mess.

## Demo

It is entirely possible that there is a demo running here: [https://taipo.robparrett.com](https://taipo.robparrett.com)

## Build

Taipo uses the [bevy 0.4](https://bevyengine.org/) engine and is super easy to build.

### Build Dependencies

- [rust stable](https://www.rust-lang.org/tools/install)
- [cargo-make](https://github.com/sagiegurari/cargo-make#installation)

### Build

```
git clone git@github.com:rparrett/taipo.git && cd taipo
cargo make serve --profile=release
```

## TODO

- [ ] Actions should optionally display their cost
  - [ ] Smaller coin sprite?
  - [ ] Cost is a function of (action, target)
- [ ] Actions should have a disabled state
  - [ ] Can't upgrade a tower if it is max-level or too expensive
  - [ ] Can't purchase a tower if it is too expensive
- [ ] A goal should exist and have hitpoints
- [ ] Enemies should appear and move towards the goal
  - [X] Walk on paths towards goal
  - [ ] Should we be running movement on a fixed timestep?
  - [ ] Pre-process paths to soften the corners (lyon?)
  - [ ] Damage the goal if we collide with it
- [X] There should be a wave countdown timer
- [X] Towers should shoot projectiles towards enemies
  - [X] Damage the enemy if the projectile collides with it
  - [X] Replace the enemy with an enemy corpse if it dies
  - [ ] Clean up those corpses
- [ ] We should load game data externally in some serialized format
  - [ ] Lexicon
  - [ ] Level
    - [X] Background Tiles
    - [X] Tower Slots
    - [X] Goal
    - [X] Enemy Spawn
    - [X] Enemy Path
    - [ ] Tower Stats
    - [ ] Enemy Waves
- [ ] Improve word parsing so hiragana/katakana are (optionally?) automatically converted to ascii
- [ ] Add a "partially typed" state to rendered glyphs?
- [ ] When sound becomes possible in Bevy/web, things should make sounds
  - [ ] Mis-typed character
  - [ ] Mis-entered action
  - [ ] Correctly entered action
  - [ ] Tower firing
  - [ ] Enemy taking damage
  - [ ] Enemy dealing damage
  - [ ] Becoming able to afford to do literally anything
- [X] Detect canvas focus and instruct player to focus canvas?
- [ ] Do an art?
  - [ ] Give up, bribe someone else to do an art
    - [ ] Train or Subway theme
- [ ] Deal with action ambiguity
  - [ ] Either prevent ambiguities when assigning words for targets
  - [ ] Or allow the player to tab through multiple completed targets
- [ ] Rewrite action display with overlapping text to fix text jitter? Will make antialiasing worse, but might be best solution until some sort of richtext exists.
- [ ] Rethink action spawning entirely to fix "back" action changing after building a tower
