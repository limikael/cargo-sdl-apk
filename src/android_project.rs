use fs_extra::{copy_items, dir::CopyOptions, remove_items};
use std::path::Path;
use std::process::Command;
use std::fs::{copy, create_dir_all, read_to_string, write};
use symlink::symlink_dir;
use std::collections::HashMap;
use crate::util::*;

pub fn build_sdl_for_android(targets: &Vec<&str>) {
    let p = Path::new(&*get_env_var("ANDROID_NDK_HOME")).join("ndk-build");

    assert!(Command::new(p)
        .args([
            "NDK_PROJECT_PATH=.",
            "APP_BUILD_SCRIPT=./Android.mk",
            "APP_PLATFORM=android-18"
        ])
        .current_dir(&*get_env_var("SDL"))
        .status()
        .unwrap()
        .success());

    for rust_name in targets {
        let android_name=get_target_android_name(rust_name);
        let rust_dir=Path::new("target").join(rust_name).join("debug/deps");

        create_dir_all(rust_dir).expect("Unable to create target dir");
        copy(
            Path::new(&*get_env_var("SDL"))
                .join("libs")
                .join(android_name)
                .join("libSDL2.so"),
            Path::new("target")
                .join(rust_name)
                .join("debug/deps/libSDL2.so")
        ).expect("Unable to copy SDL dependencies");
    }
}

pub fn get_target_android_name(rust_target_name: &str)->&str {
    match rust_target_name {
        "aarch64-linux-android"=>"arm64-v8a",
        "armv7-linux-androideabi"=>"armeabi-v7a",
        "i686-linux-android"=>"x86",
        _=>{panic!("Unknown target: {}",rust_target_name)}
    }
}

pub fn get_android_app_id(manifest_path: &Path)->String {
    get_toml_string(manifest_path,
        vec!["package","metadata","android","package_name"]
    ).unwrap_or("org.libsdl.app".to_string())
}

fn create_android_project(
        manifest_path: &Path, 
        target_artifacts: &HashMap<String,String>) {
    let manifest_dir=manifest_path.parent().unwrap();
    let appid=get_android_app_id(manifest_path);

    let appname=get_toml_string(manifest_path,
        vec!["package","metadata","android","title"]
    ).unwrap_or("Untitled".to_string());

    // Copy template project from SDL
    copy_items(
        &[Path::new(&*get_env_var("SDL")).join("android-project")],
        Path::new(manifest_dir).join("target"),
        &CopyOptions::new().skip_exist(true),
    )
    .unwrap();

    // Create main activity class
    let java_main_folder=manifest_dir
        .join("target/android-project/app/src/main/java")
        .join(str::replace(&appid, ".", "/"));
    create_dir_all(java_main_folder.clone()).unwrap();
    let main_class = "
		package $APP;

		import org.libsdl.app.SDLActivity;

		public class MainActivity extends SDLActivity {
		}
	";
    let main_class = str::replace(main_class, "$APP", &appid);
    write(java_main_folder.join("MainActivity.java"), &main_class).expect("Unable to write file");

    // Change project files
    change_android_project_file(
        manifest_dir,
        "app/src/main/AndroidManifest.xml",
        vec![("SDLActivity", "MainActivity"), ("org.libsdl.app", &*appid)],
    );

    change_android_project_file(
        manifest_dir,
        "app/build.gradle",
        vec![("org.libsdl.app", &*appid)]
    );

    change_android_project_file(
        manifest_dir,
        "app/src/main/res/values/strings.xml",
        vec![("Game", &*appname)],
    );

    // Remove C sources
    remove_items(&[
        manifest_dir.join("target/android-project/app/jni/src")
    ]).unwrap();

    // Link SDL into project
    if !manifest_dir.join("target/android-project/app/jni/SDL").is_dir() {
        symlink_dir(
            Path::new(&*get_env_var("SDL")),
            manifest_dir.join("target/android-project/app/jni/SDL"),
        )
        .unwrap();
    }

    // Copy libmain.so to all targets
    for (target, artifact) in target_artifacts {
        let target_android_name=get_target_android_name(target);
        //println!("{:?}",target);

        let android_dir = manifest_dir
            .join("target/android-project/app/src/main/jniLibs")
            .join(target_android_name);

        create_dir_all(&android_dir).unwrap();
        copy(
            artifact,
            android_dir.join("libmain.so")
        ).unwrap();
    }
}

fn change_android_project_file(manifest_dir: &Path, file_name: &str, replacements: Vec<(&str, &str)>) {
    let mut content = read_to_string(Path::new(&*get_env_var("SDL"))
        .join("android-project")
        .join(file_name)
    ).expect("Unable to read manifest file");

    for (from, to) in replacements {
        content = content.replace(from, to);
    }

    //println!("{:?}",manifest_dir.join("target/android-project").join(file_name));

    write(
        manifest_dir.join("target/android-project").join(file_name),
        &content,
    )
    .expect("Unable to write file");
}

pub fn build_android_project(
        manifest_path: &Path, 
        target_artifacts: &HashMap<String,String>) {
    let manifest_dir=manifest_path.parent().unwrap();

    create_android_project(manifest_path,target_artifacts);

    assert!(Command::new("./gradlew")
        .args(["assembleDebug"])
        .current_dir(manifest_dir.join("./target/android-project"))
        .status()
        .unwrap()
        .success());
}
