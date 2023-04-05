# cargo-sdl-apk
Build Android packages that uses SDL.

This cargo tool aims to be for [Rust SDL](https://docs.rs/sdl2/latest/sdl2/) what
[cargo-apk](https://crates.io/crates/cargo-apk) is for [Glutin](https://crates.io/crates/glutin), 
and [cargo-quad-apk](https://crates.io/crates/cargo-quad-apk) is for [Miniquad](https://crates.io/crates/miniquad).

That is, a simple command to package up an APK and upload it to your phone and start it.

The way it works internally is by automating the steps described 
[in this article](https://julhe.github.io/posts/building-an-android-app-with-rust-and-sdl2/) by Julian Heinken.

I created it as a tool for my UI library [Appy](https://github.com/limikael/appy), but it can be used for other Rust SDL projects as well.

It currently suffers from complete lack of documentation and some wierd assumptions made, such as you need to call the app you are building "main".

But basically the way to use it is:

1. Set the environment variables:
   * `ANDROID_HOME` pointing to the Android SDK.
   * `ANDROID_NDK_HOME` pointing to the Android NDK.
   * `SDL` pointing to the SDL source dir.

2. Run `cargo sdl-apk build` or `cargo sdl-apk run`.
