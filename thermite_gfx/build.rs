use shaderc;
use std::env;
use std::fs::{self, DirBuilder};
use std::path::{Path, PathBuf};
use walkdir::{self, WalkDir};

fn main() {
    // Tell the build script to only run again if we change our source shaders
    println!("cargo:rerun-if-changed=assets/shaders");
    // Grab our environment variables to we can place artifacts in the correct places
    // The directory where this build script places artifacts
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Couldn't get OUT_DIR envVar"));
    // The directory containing the manifest (cargo.toml) of this package
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Couldn't get CARGO_MANIFEST_DIR envVar"),
    );

    // Since we already fenced this build script to run only if changes to shaders are made, we can always run this
    cross_compile_glsl_shaders_to_spirv();

    // Locate executable path even if the project is in workspace
    let executable_path = locate_target_dir_from_output_dir(&out_dir)
        .expect("failed to find target dir")
        .join(env::var("PROFILE").expect("Couldn't get PROFILE envVar"));

    // Copy all relevant build artifacts to the output folder for usage
    copy(
        &manifest_dir.join("assets"),
        &executable_path.join("assets"),
    );
}

fn locate_target_dir_from_output_dir(mut target_dir_search: &Path) -> Option<&Path> {
    loop {
        // if path ends with "target", we assume this is correct dir
        if target_dir_search.ends_with("target") {
            return Some(target_dir_search);
        }

        // otherwise, keep going up in tree until we find "target" dir
        target_dir_search = match target_dir_search.parent() {
            Some(path) => path,
            None => break,
        }
    }
    None
}

fn copy(from: &Path, to: &Path) {
    let from_path: PathBuf = from.into();
    let to_path: PathBuf = to.into();
    for entry in WalkDir::new(from_path.clone()) {
        let entry = entry.unwrap();

        if let Ok(rel_path) = entry.path().strip_prefix(&from_path) {
            let target_path = to_path.join(rel_path);

            if entry.file_type().is_dir() {
                DirBuilder::new()
                    .recursive(true)
                    .create(target_path)
                    .expect("failed to create target dir");
            } else {
                fs::copy(entry.path(), &target_path).expect("failed to copy");
            }
        }
    }
}

fn cross_compile_glsl_shaders_to_spirv() {
    // Create our shader cross-compiler
    let mut compiler = shaderc::Compiler::new().expect("Could not create glsl->spirv compiler");
    let options =
        shaderc::CompileOptions::new().expect("Could not create glsl->spirv compiler options"); // Can alter compiler options here

    // Create a glsl->spirv destination path if neccessary
    fs::create_dir_all("assets/shaders/spirv_out").expect("Couldn't create SPIR-V output dir");

    // Loop over all glsl shaders to cross-compile them to spir-v format
    for entry in fs::read_dir("assets/shaders").expect("Cannot read dir: assets/shaders") {
        let entry: fs::DirEntry = entry.expect("Couldn't grab direntry");
        if entry
            .file_type()
            .expect("Could not get file type, probably a symlink")
            .is_file()
        {
            let path = entry.path();
            let filename = entry
                .file_name()
                .into_string()
                .expect("Could not grab proper filename");
            let shader_type =
                path.extension()
                    .and_then(|ext| match ext.to_string_lossy().as_ref() {
                        "vert" => Some(shaderc::ShaderKind::Vertex),
                        "frag" => Some(shaderc::ShaderKind::Fragment),
                        _ => None,
                    });
            if let Some(shader_type) = shader_type {
                let source =
                    fs::read_to_string(&path).expect("Couldn't read source code from shader");
                let compilation_result = compiler.compile_into_spirv(
                    &source,
                    shader_type,
                    &filename,
                    "main",
                    Some(&options),
                );
                match compilation_result {
                    Result::Ok(compiled_spirv) => {
                        let num_warnings = compiled_spirv.get_num_warnings();
                        let warning_msgs = compiled_spirv.get_warning_messages();
                        println!(
                            "{} GLSL -> SPIR-V cross-compilation succeeded with {} warnings:\n{}",
                            filename, num_warnings, warning_msgs
                        );
                        let compiled_bytes = compiled_spirv.as_binary_u8();
                        let out_path = format!("assets/shaders/spirv_out/{}.spv", filename);
                        fs::write(&out_path, &compiled_bytes)
                            .expect("Couldn't write compiled SPIR-V shader to output dir");
                    }
                    Result::Err(err) => {
                        panic!(
                            "{} GLSL -> SPIR-V cross-compilation failed:\n{}",
                            filename, err
                        );
                    }
                }
            }
        }
    }
}
