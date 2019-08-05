# Changelog

All notable changes to **[Rusty Rockets]** are documented in this file.

The format is based on [Keep a Changelog], and this project adheres to [Semantic
Versioning]. The file is auto-generated using [Conventional Commits].

## [Unreleased]

_nothing new to show for… yet!_

## [0.2.0]

- date: 2019-08-05
- name: GFX Tutorial, Part II

### Description

Another quick prototype release.

Aside from going through the paces of the GFX tutorial, this release also adds
an experimental embedded game loop library to further learn about the concepts
related to decoupled rendering, and a fixed time step.

It doesn't do anything useful yet, aside from being a well-documented training
ground.

![Screenshot #1](https://github.com/rustic-games/prototype/raw/master/assets/marketing/releases/v0.2.0/screenshot.png?sanitize=true)

### Contributions

This release was made possible by the following people (in alphabetical order).
Thank you all for your contributions. Your work – no matter how significant – is
greatly appreciated by the community. ❤️

- [Jean Mertz] \(<jean@mertz.fm>\)

### Changes

#### Game Features

- **add initial game loop implementation** [07ebc32]

  This adds a simple (but well-documented) "game loop" library to the project.

  It's mainly meant for learning purposes, and to see what should, and shouldn't
  be abstracted away in a library.

- **"Learn gfx-hal" chapter 4** [d4c4880]

  This implements the fourth chapter of the "Learn gfx-hal" tutorial[0].

  Some take-aways:

  - Still same feeling as before, lots of boilerplate code, lots of unsafe
    code. But, having fun going through the courses.

  - Updating the code to use gfx-hal 0.2, contributing the changes back
    upstream[1].

  - Also ordered a book[2], and started reading a free online book[3] to
    brush up on the required knowledge.

  [0]: https://lokathor.github.io/learn-gfx-hal/
  [1]: https://github.com/Lokathor/learn-gfx-hal/pull/86
  [2]: https://www.goodreads.com/book/show/43299232
  [3]: https://paroj.github.io/gltut/index.html

#### _Unchanged_

_The following categories contain no changes in this release: bug fixes,
documentation, tests, performance improvements, refactoring, code styling._

## [0.1.0]

- date: 2019-08-02
- name: GFX Tutorial, Part I

### Description

The first in a long sequence of _prototyping releases_. Every release will
change a tiny bit of the application, starting with nothing related to the game
at all, slowly building towards something that takes on a familiar shape,
representing what is described in the design documentation.

In the first couple of releases, the GFX tutorial is completed step by step, to
get a feel for the low-level graphics programming requirements.

![Screenshot #1](https://github.com/rustic-games/prototype/raw/master/assets/marketing/releases/v0.1.0/screenshot.png?sanitize=true)

### Contributions

This release was made possible by the following people (in alphabetical order).
Thank you all for your contributions. Your work – no matter how significant – is
greatly appreciated by the community. ❤️

- [Jean Mertz] \(<jean@mertz.fm>\)

### Changes

#### Game Features

- **"Learn gfx-hal" chapters 1, 2 & 3** [fa74d27]

  This implements the first three chapters of the "Learn gfx-hal" tutorial[0].

  Some take-aways:

  - `gfx-hal` is low-level, requiring lots of boilerplate code. It will be
    interesting to see the difference once we start migrating the code to
    the `Rendy`[1] crate.

  - Lots of unsafe code required.

  - The `winit`[2] crate is reasonably heavy, providing platform support that
    isn't relevant to the game (at least not yet).

  - Both crates are still young, lots of moving targets.

  [0]: lokathor.github.io/learn-gfx-hal
  [1]: rustgd/rendy
  [2]: rust-windowing/winit

#### _Unchanged_

_The following categories contain no changes in this release: bug fixes,
documentation, tests, performance improvements, refactoring, code styling._

<!-- [contributors] -->

[jean mertz]: https://github.com/JeanMertz

<!-- [releases] -->

[unreleased]: https://github.com/rustic-games/prototype/compare/v0.1.0...HEAD
[0.2.0]: https://github.com/rustic-games/prototype/releases/tag/v0.2.0
[0.1.0]: https://github.com/rustic-games/prototype/releases/tag/v0.1.0

<!-- [commits] -->

[07ebc32]: https://github.com/rustic-games/prototype/commit/07ebc32158a31df795ea4bd05588e39c1ced1fef
[fa74d27]: https://github.com/rustic-games/prototype/commit/fa74d27da60e78f4a8c85b86a4a431ccf7b43210

<!-- [references] -->

[rusty rockets]: https://rustic.games/
[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
[conventional commits]: https://www.conventionalcommits.org/en/v1.0.0-beta.4/
