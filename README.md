# LearnOpenGL Rust

Rewrite of LearnOpenGL example source code in Rust language.

# Instructions on how to build this repository

## Prerequisites

First of all you need Rust installed on your computer. Using [rustup](https://rustup.rs/) is recommended.

Once you've set up Rust building environment, there are some libraries required to be installed,
most of which are already mentioned within the [official LearnOpenGL documentation](https://learnopengl.com/)
about how to get them installed;  here the links are given below as well so that you can get access to these
resources at once:

[GLFW](https://www.glfw.org/docs/latest/compile.html)

[Assimp](http://assimp.org/index.php/downloads)

[FreeType](http://www.freetype.org/)

# Build the source code

To build and run test, switch to the repository root directory and run:

```shell
cargo test
```

To build and run example code for certain section of LearnOpenGL, switch to that directory and run:
```shell
cargo run
```

For example, to run example code within the "Hello Triangle" section of "Getting started" chapter,
execute in shell as the following (assuming you are in the repository root directory initially):

```shell
cd crates/1.getting_started/2.1.hello_triangle
cargo run
```

# License

For the images, audios and models used under this repository (under the `resources` directory), licensed under the
[CC BY 4.0](https://spdx.org/licenses/CC-BY-4.0.html) license as mentioned in
[LearnOpenGL About](https://learnopengl.com/About) page.

For the other parts, licensed under [the Apache License v2.0](https://spdx.org/licenses/Apache-2.0.html).
See [LICENSE.txt](LICENSE.txt) for details.