use std::fs::canonicalize;
use std::process::Command;
use std::path::Path;

mod build_bin_as_lib;
use build_bin_as_lib::*;

mod util;
use util::*;

mod android_project;
use android_project::*;

const HELP: &str = "
cargo-sdl-apk -- Build APKs with Rust and SDL.

USAGE:
  cargo sdl-apk <command> [OPTIONS] 

COMMANDS:
  build                 Builds APK from bin target.
  run                   Build APK and run using adb.

OPTIONS:
  --manifest-path PATH  Path to Cargo.toml.
  --example EXAMPLE     Build or run crate example.
";

#[derive(Debug)]
struct SdlApkArgs {
    manifest_path: String,
    command: String,
    example: Option<String>
}

fn parse_args()->Result<SdlApkArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    let mut cmd=pargs.free_from_str()?;
    if cmd=="sdl-apk" {
        cmd=pargs.free_from_str()?;
    }

    let args=SdlApkArgs {
        manifest_path: pargs.value_from_str("--manifest-path").unwrap_or("Cargo.toml".to_string()),
        example: pargs.opt_value_from_str("--example")?,
        command: cmd
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        return Err(pico_args::Error::ArgumentParsingFailed{
            cause: format!("Unknown arguments: {:?}.", remaining)
        });
    }

    Ok(args)
}

fn build_android(manifest_path: &Path, build_target:BuildTarget) {
    for k in &["ANDROID_HOME", "ANDROID_NDK_HOME", "SDL"] {
        let _check_val = get_env_var(k);
    }

    let targets=vec![
    	"aarch64-linux-android",
    	"armv7-linux-androideabi",
    	"i686-linux-android"
    ];

    build_sdl_for_android(&targets);
    let target_artifacts=build_bin_as_lib(&manifest_path,build_target,&targets);
    build_android_project(&manifest_path,&target_artifacts);
}

fn run_android(manifest_path: &Path, build_target:BuildTarget) {
    build_android(manifest_path,build_target);

    let appid = get_android_app_id(manifest_path);

    let p = Path::new(&*get_env_var("ANDROID_HOME")).join("platform-tools/adb");
    assert!(Command::new(p.clone())
        .args([
            "-d",
            "install",
            "-r",
            "target/android-project/app/build/outputs/apk/debug/app-debug.apk"
        ])
        .status()
        .unwrap()
        .success());

    assert!(Command::new(p.clone())
        .args(["shell", "am", "force-stop", &*appid])
        .status()
        .unwrap()
        .success());

    let mut activity = appid.clone();
    activity.push_str("/.MainActivity");

    assert!(Command::new(p.clone())
        .args(["shell", "am", "start", "-W", "-n", &*activity])
        .status()
        .unwrap()
        .success());

    let pid_vec = Command::new(p.clone())
        .arg("shell")
        .arg("pidof")
        .arg(&*appid)
        .output()
        .unwrap() //except("Can't get pid")
        .stdout;

    let pid = std::str::from_utf8(&pid_vec).unwrap().trim();
    let pid: u32 = pid
        .parse()
        .unwrap();

    println!("Launched with PID: {}", pid);

    assert!(Command::new(p.clone())
        .args(["logcat","-v","color","--pid",&*pid.to_string()])
        .status()
        .unwrap()
        .success());
}

fn main() {
    let args=match parse_args() {
        Ok(v)=>v,
        Err(e)=>{
            eprintln!("Error: {}.", e);
            println!("{}",HELP);
            std::process::exit(1);
        }
    };

    let manifest_path=canonicalize(args.manifest_path).unwrap();

    let build_target=match &args.example {
        None=>BuildTarget::Bin,
        Some(s)=>BuildTarget::Example(s.clone())
    };

    match &*args.command {
        "build"=>build_android(&manifest_path,build_target),
        "run"=>run_android(&manifest_path,build_target),
        _=>{
            eprintln!("Unknown command: {}.", args.command);
            println!("{}",HELP);
            std::process::exit(1);
        }
    }
}