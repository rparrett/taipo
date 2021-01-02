# taipo (working name)

Taipo is an experimental game exploring the idea of limiting control of the game to typing random Japanese phrases.

This could turn out to be a useful tool to practice quick Japanese recognition / production.

Currently targeting wasm/webgl only, but there's no reason we couldn't do native desktop builds too.

## Demo

It is entirely possible that there is a demo running here: [https://taipo.robparrett.com](https://taipo.robparrett.com)

## Build

### Build Dependencies

rust (stable)

cargo-make

```
cargo install --force cargo-make
```

### Build

```
git clone git@github.com:rparrett/taipo.git
cd taipo
cargo make serve
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
  - [ ] Should we be running movement on a fixed timestep?
  - [ ] Add lyon into the mix?
  - [ ] Pre-process paths to soften the corners
  - [ ] Walk those paths
  - [ ] Damage the goal if we collide with it
- [ ] There should be a wave countdown timer
- [ ] Towers should shoot projectiles towards enemies
  - [ ] Damage the enemy if the projectile collides with it
  - [ ] Replace the enemy with an enemy corpse if it dies
- [ ] We should load game data externally in some serialized format
  - [ ] Lexicon
  - [ ] Level
    - [X] Background Tiles
    - [X] Tower Slots
    - [ ] Goal
    - [ ] Enemy Spawn
    - [ ] Enemy Path
    - [ ] Enemy Waves
- [ ] Improve word parsing so hiragana/katakana are (optionally?) automatically converted to ascii
- [ ] When sound becomes possible in Bevy/web, things should make sounds
  - [ ] Mis-typed character
  - [ ] Mis-entered action
  - [ ] Correctly entered action
  - [ ] Tower firing
  - [ ] Enemy taking damage
  - [ ] Enemy dealing damage
- [X] Detect canvas focus and instruct player to focus canvas?
- [ ] Do an art?
  - [ ] Give up, bribe someone else to do an art
    - [ ] Train or Subway theme
- [ ] If multiple actions match the input, use the longest or first.
