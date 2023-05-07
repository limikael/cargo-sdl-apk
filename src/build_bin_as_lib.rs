use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use cargo::CargoResult;
use cargo::core::{Workspace, Target, PackageId, TargetKind};
use cargo::core::resolver::CliFeatures;
use cargo::core::compiler::{Executor, BuildConfig, CompileMode, CompileTarget, CompileKind};
use cargo::ops::{FilterRule, LibRule, CompileOptions, CompileFilter, Packages};
use cargo::util::Config as CargoConfig;
use cargo_util::{ProcessBuilder};
use std::sync::Mutex;
use crate::util::*;

fn get_target_linker(rust_target_name: &str)->&str {
    match rust_target_name {
        "aarch64-linux-android"=>"toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android26-clang",
        "armv7-linux-androideabi"=>"toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi26-clang",
        "i686-linux-android"=>"toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android26-clang",
        _=>{panic!("Unknown target: {}",rust_target_name)}
    }
}

pub struct LibExecutor {
    linkers: HashMap<String,String>,
    out: Arc<Mutex<HashMap<String,String>>>
}

impl LibExecutor {
    pub fn new(linkers:HashMap<String,String>)->Self {
        Self {
            linkers,
            out: Arc::new(Mutex::new(HashMap::new()))
        }
    }
}

impl Executor for LibExecutor {
    fn exec(
        &self,
        cmd: &ProcessBuilder,
        _id: PackageId,
        target: &Target,
        mode: CompileMode,
        on_stdout_line: &mut dyn FnMut(&str) -> CargoResult<()>,
        on_stderr_line: &mut dyn FnMut(&str) -> CargoResult<()>,
    ) -> CargoResult<()> {
        if mode == CompileMode::Build
                && (target.kind() == &TargetKind::Bin || target.kind() == &TargetKind::ExampleBin) {
            let args:Vec<&str>=cmd.get_args().map(|s|s.to_str().unwrap()).collect();
            let mut new_args=vec![];
            let mut i=0;
            let mut out_dir="".to_string();
            let mut target_triple="".to_string();
            while i<args.len() {
                if args[i]=="--crate-type" 
                        && args[i+1]=="bin" {
                    new_args.push("--crate-type");
                    new_args.push("dylib");
                    i+=2;
                }

                else if args[i]=="--out-dir" {
                    out_dir=args[i+1].to_string();
                    new_args.push(args[i]);
                    new_args.push(args[i+1]);
                    i+=2;
                }

                else if args[i]=="--target" {
                    target_triple=args[i+1].to_string();
                    new_args.push(args[i]);
                    new_args.push(args[i+1]);
                    i+=2;
                }

                else {
                    new_args.push(args[i]);
                    i+=1;
                }
            }

            let mut linker_arg="linker=".to_string();
            linker_arg.push_str(&*self.linkers.get(&target_triple).unwrap());

            new_args.push("-C");
            new_args.push(&*linker_arg);

            //println!("the new args: {:?}",new_args.join(" "));

            let mut cmd = cmd.clone();
            cmd.args_replace(&new_args);
            cmd.exec_with_streaming(on_stdout_line, on_stderr_line, false)
                .map(drop)?;

            let stdout=cmd.arg("--print").arg("file-names").exec_with_output()?;
            let stdout=String::from_utf8(stdout.stdout).unwrap();
            let stdout=stdout.lines().next().unwrap();
            let p=Path::new(&*out_dir).join(stdout.clone());
            let p=p.into_os_string().into_string().unwrap();

            self.out.lock().unwrap().insert(target_triple,p);
        }

        else {
            cmd.exec_with_streaming(on_stdout_line, on_stderr_line, false)
                .map(drop)?;
        }

        Ok(())
    }
}

pub enum BuildTarget {
    Bin,
    Example(String)
}

pub fn build_bin_as_lib(
        manifest_path:&Path,
        build_target: BuildTarget,
        targets:&Vec<&str>,
    )->HashMap<String,String> {
    let mut linkers:HashMap<String,String>=HashMap::new();
    for t in targets {
        linkers.insert(
            t.to_string(),
            Path::new(&*get_env_var("ANDROID_NDK_HOME"))
                .join(get_target_linker(t))
                .into_os_string().into_string().unwrap()
        );
    }

    let mut cargo_config = CargoConfig::default().unwrap();
    cargo_config.configure(
    	0, // verbose
    	false, // quiet
    	None, // color
    	cargo_config.frozen(), // frozen
    	cargo_config.locked(), // locked
    	cargo_config.offline(), // offline
    	&None, // target dir
    	&[], // unstable flags
    	&[] // cli config
    ).unwrap();

    let workspace = Workspace::new(manifest_path, &cargo_config).unwrap();

    let mut build_config=BuildConfig::new(
    	&cargo_config,
    	None,
    	false,
    	&[],
    	CompileMode::Build
    ).unwrap();

    build_config.requested_kinds=targets.iter().map(|s|{
        CompileKind::Target(CompileTarget::new(s).unwrap())
    }).collect();

    let compile_options=CompileOptions {
        build_config: build_config,
        cli_features: CliFeatures::new_all(false),
        spec: Packages::Packages(Vec::new()),
        filter: CompileFilter::Only {
        	all_targets: false,
        	lib: LibRule::False,
            bins: match build_target {
                BuildTarget::Bin=>FilterRule::All,
                BuildTarget::Example(_)=>FilterRule::Just(vec![])
            },
        	examples: match build_target {
                BuildTarget::Bin=>FilterRule::Just(vec![]),
                BuildTarget::Example(s)=>FilterRule::Just(vec![s])
            },
        	tests: FilterRule::Just(vec![]),
        	benches: FilterRule::Just(vec![]),
        },
        target_rustdoc_args: None,
        target_rustc_args: None,
        target_rustc_crate_types: None,
        rustdoc_document_private_items: false,
        honor_rust_version: true,
    };

    let executor=Arc::new(LibExecutor::new(linkers));
    let executor_dyn:Arc<dyn Executor>=executor.clone();
    cargo::ops::compile_with_exec(&workspace,&compile_options,&executor_dyn).unwrap();

    let out=&*executor.out.lock().unwrap();
    out.clone()
}