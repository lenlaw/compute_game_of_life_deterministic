# compute_game_of_life_deterministic
A deterministic implementation of Conway's Game Of Life using shader compute.

# Description

This implementation is heavily based on the [Official Bevy example of Conway's Game Of Life](https://github.com/bevyengine/bevy/blob/a31ebdc1a68c1782a18d2224133d10e889800485/examples/shader/compute_shader_game_of_life.rs). However, the official Bevy example is not Conway's Game of Life due to it being non-deterministic. In Conway's Game Of Life the following is true: 

'The first generation is created by applying the above rules simultaneously to every cell in the seed, live or dead; births and deaths occur simultaneously, and the discrete moment at which this happens is sometimes called a tick.[nb 1] Each generation is a pure function of the preceding one. The rules continue to be applied repeatedly to create further generations.' [Conway's Game Of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life#Rules)

In the official Bevy example the cells of the game world are updated ad-hoc as shader invocations execute and complete in an unplanned order. This results in the game world changing with each completed execution of a shader invocation, rather than rules being applied 'simultaneously to every cell'. Consequently there is non-deterministic behaviour.

This implementation uses an extra _write-texture_ to hold the changing world whilst keeping the _read-texture_ unchanged as the Conway's rules are applied. After all invocations have completed, the write-texture is copied to the _read-texture_ by an additional shader pipeline before Bevy renders the game world.

An additional minor change has been made to the initial conditions for the game. The official example sets cells as live or dead using a random shader function. This implementation replaces the random initialisation with a simple block of live cells. The block allows for the confirmation of determinisism in the game, as the same initial conditions can be seen to produce the same outcomes. 

## License

Bevy is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.

Some of the engine's code carries additional copyright notices and license terms due to their external origins.
These are generally BSD-like, but exact details vary by crate:
If the README of a crate contains a 'License' header (or similar), the additional copyright notices and license terms applicable to that crate will be listed.
The above licensing requirement still applies to contributions to those crates, and sections of those crates will carry those license terms.
The [license](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields) field of each crate will also reflect this.
For example, [`bevy_mikktspace`](./crates/bevy_mikktspace/README.md#license-agreement) has code under the Zlib license (as well as a copyright notice when choosing the MIT license).

The [assets](assets) included in this repository (for our [examples](./examples/README.md)) typically fall under different open licenses.
These will not be included in your game (unless copied in by you), and they are not distributed in the published bevy crates.
See [CREDITS.md](CREDITS.md) for the details of the licenses of those files.
