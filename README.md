# cargo-sdl-apk
Build Android packages that use SDL.

This cargo tool aims to be for [Rust SDL](https://docs.rs/sdl2/latest/sdl2/) what
[cargo-apk](https://crates.io/crates/cargo-apk) is for [Glutin](https://crates.io/crates/glutin), 
and [cargo-quad-apk](https://crates.io/crates/cargo-quad-apk) is for [Miniquad](https://crates.io/crates/miniquad). That is, a simple command to package up an APK and upload it to your phone and start it. The way it works internally is by automating the steps described 
[in this article](https://julhe.github.io/posts/building-an-android-app-with-rust-and-sdl2/) by Julian Heinken. I created it as a tool for my UI library [Appy](https://github.com/limikael/appy), but it can be used for other Rust SDL projects as well.

## Basic usage

1. Install with `cargo install cargo-sdl-apk`.
2. Make sure you have the following:
   * The SDL source, clone it from [here](https://github.com/libsdl-org/SDL). Make sure you have the `release-2.26.x` branch.
   * Java. Muse be jdk17 (doesn't work with jdk19).
   * Android SDK with command line tools.
   * Android NDK.
3. Set the environment variables:
   * `ANDROID_HOME` pointing to the Android SDK.
   * `ANDROID_NDK_HOME` pointing to the Android NDK.
   * `SDL` pointing to the SDL source dir.
4. Run `cargo sdl-apk build` or `cargo sdl-apk run` from inside your SDL application crate. You can also use
   `cargo sdl-apk run --example some_example` to run a crate example, in a similar way as you would do with
   cargo.

## Project setup

The entry point for your application must be called `SDL_main` and use the attribute `#[no_mangle]`. Here is
an [example project](https://github.com/limikael/cargo-sdl-apk/tree/master/example-project). To build/run the
project, cd into it and run `cargo sdl-apk build` or `cargo sdl-apk run`.
