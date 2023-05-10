use fs_extra::{copy_items, dir::CopyOptions, remove_items};
use std::path::Path;
use std::process::Command;
use std::fs::{copy, create_dir_all, read_to_string, write};
use symlink::symlink_dir;
use std::collections::HashMap;
use crate::util::*;
use crate::BuildProfile;

pub fn build_sdl_for_android(targets: &Vec<&str>, profile:BuildProfile) {
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
        let rust_dir=Path::new("target")
            .join(rust_name)
            .join(profile.to_string())
            .join("deps");

        create_dir_all(rust_dir).expect("Unable to create target dir");
        copy(
            Path::new(&*get_env_var("SDL"))
                .join("libs")
                .join(android_name)
                .join("libSDL2.so"),
            Path::new("target")
                .join(rust_name)
                .join(profile.to_string())
                .join("deps/libSDL2.so")
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

pub fn sign_android(
        manifest_path: &Path, 
        ks_file: Option<String>,
        ks_pass: Option<String>
    ) {
    let manifest_dir=manifest_path.parent().unwrap();
    let release_dir=manifest_dir.join("target/android-project/app/build/outputs/apk/release");
    //println!("{:?}",release_dir);

    // Find android build tools.
    let tool_paths=std::fs::read_dir(Path::new(&*get_env_var("ANDROID_HOME")).join("build-tools")).unwrap();
    let mut tool_paths:Vec<String>=tool_paths.map(|d|{
        d.unwrap().path().file_name().unwrap()
            .to_os_string().into_string().unwrap()
    }).collect();
    tool_paths.sort();
    tool_paths.reverse();
    let tools_version=tool_paths[0].clone();
    println!("Using build-tools: {}",tools_version);

    // Determine key file. Generate if needed.
    let (key_file,key_pass)=if ks_file.is_some() {
        (
            ks_file.unwrap(),
            ks_pass.expect("Need keystore password")
        )
    } else {
        let key_path=release_dir.join("app-release.jks");
        if !key_path.exists() {
            println!("Generating keyfile...");
            assert!(Command::new("keytool")
                .arg("-genkey")
                .arg("-dname").arg("CN=Unknown, OU=Unknown, O=Unknown, L=Unknown, S=Unknown, C=Unknown")
                .arg("-storepass").arg("android")
                .arg("-keystore").arg(key_path.clone())
                .arg("-keyalg").arg("RSA")
                .arg("-keysize").arg("2048")
                .arg("-validity").arg("10000")
                .status()
                .unwrap()
                .success());
        }

        (
            key_path.into_os_string().into_string().unwrap(),
            "pass:android".to_string()
        )
    };

    println!("Using keyfile: {}",key_file);

    // Run zipalign.
    let zipalign_path=Path::new(&*get_env_var("ANDROID_HOME"))
        .join("build-tools").join(tools_version.clone()).join("zipalign");

    assert!(Command::new(zipalign_path)
        .arg("-v").arg("-f")
        .arg("-p").arg("4")
        .arg(release_dir.join("app-release-unsigned.apk"))
        .arg(release_dir.join("app-release-unsigned-aligned.apk"))
        .status()
        .unwrap()
        .success());

    // Run apksigner
    let apksigner_path=Path::new(&*get_env_var("ANDROID_HOME"))
        .join("build-tools").join(tools_version.clone()).join("apksigner");

    assert!(Command::new(apksigner_path)
        .arg("sign")
        .arg("-ks").arg(key_file)
        .arg("-ks-pass").arg(key_pass)
        .arg("-out").arg(release_dir.join("app-release.apk"))
        .arg(release_dir.join("app-release-unsigned-aligned.apk"))
        .status()
        .unwrap()
        .success());
}

// keytool -android blabla -genkey -v -keystore my-release-key.jks -keyalg RSA -keysize 2048 -validity 10000 -alias my-alias
// /home/micke/Android/Sdk/build-tools/30.0.3/zipalign -v -p 4 app-release-unsigned.apk app-release-unsigned-aligned.apk
// /home/micke/Android/Sdk/build-tools/30.0.3/apksigner sign -ks my-release-key.jks -ks-pass pass:android -out app-release.apk app-release-unsigned-aligned.apk

pub fn build_android_project(
        manifest_path: &Path, 
        target_artifacts: &HashMap<String,String>,
        profile:BuildProfile,
        ks_file: Option<String>,
        ks_pass: Option<String>
    ) {
    let manifest_dir=manifest_path.parent().unwrap();

    create_android_project(manifest_path,target_artifacts);

    let gradle_task=match profile {
        BuildProfile::Debug=>"assembleDebug",
        BuildProfile::Release=>"assembleRelease",
    };

    assert!(Command::new("./gradlew")
        .args([gradle_task])
        .current_dir(manifest_dir.join("./target/android-project"))
        .status()
        .unwrap()
        .success());

    if matches!(profile,BuildProfile::Release) {
        sign_android(manifest_path,ks_file,ks_pass);
    }
}
