use sdl2::event::Event;

// The entry point for our application must be caled SDL_main, and
// must be attributed #[no_mangle]. From within this function we can
// call out regular main function. This way the same program can run
// both on desktop and on Android.
#[no_mangle]
#[allow(non_snake_case)]
pub fn SDL_main() {
    main();
}

pub fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem
        .window("Testing SDL", 800, 600)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        gl::ClearColor(0.0, 0.5, 0.75, 0.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
    window.gl_swap_window();

    let mut event_pump = sdl.event_pump().unwrap();
    loop {
        match event_pump.wait_event() {
            Event::Quit { .. } | Event::KeyDown { .. } => std::process::exit(0),
            _ => {}
        }
    }
}
