# cargo-sdl-apk
Build Android packages that uses SDL.

This cargo tool aims to be for [Rust SDL](https://docs.rs/sdl2/latest/sdl2/) what
[cargo-apk](https://crates.io/crates/cargo-apk) is for [Glutin](https://crates.io/crates/glutin), 
and [cargo-quad-apk](https://crates.io/crates/cargo-quad-apk) is for [Miniquad](https://crates.io/crates/miniquad).

That is, a simple command to package up an APK and upload it to your phone and start it.

The way it works internally is by automating the steps described 
[in this article](https://julhe.github.io/posts/building-an-android-app-with-rust-and-sdl2/) by Julian Heinken.

I created it as a tool for my UI library [Appy](https://github.com/limikael/appy), but it can be used for other Rust SDL projects as well.

It currently suffers from complete lack of documentation.

The lib target you are building must have its default name, i.e. same as the package name.

Basic usage:

1. It is not published as a crate (yet). Install it from GitHub with:
   ```
   cargo install --git https://github.com/limikael/cargo-sdl-apk.git
   ```
2. Make sure you have the following:
   * The SDL source, clone it from [here](https://github.com/libsdl-org/SDL). Make sure you have the `release-2.26.x` branch.
   * Java. Muse be jdk17 (doesn't work with jdk19).
   * Android SDK with command line tools.
   * Android NDK.
3. Set the environment variables:
   * `ANDROID_HOME` pointing to the Android SDK.
   * `ANDROID_NDK_HOME` pointing to the Android NDK.
   * `SDL` pointing to the SDL source dir.

4. Run `cargo sdl-apk build` or `cargo sdl-apk run` from inside your SDL application crate.
