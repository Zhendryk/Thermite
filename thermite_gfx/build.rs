use shaderc;
use spirv_cross::{hlsl, msl, spirv, ErrorCode};
use std::env;
use std::fs::{self, DirBuilder};
use std::path::{Path, PathBuf};
use walkdir::{self, WalkDir};

fn main() {
    // Tell the build script to only run again if we change our source shaders
    println!("cargo:rerun-if-changed=assets/shaders/");
    // Grab our environment variables to we can place artifacts in the correct places
    // The directory where this build script places artifacts
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Couldn't get OUT_DIR envVar"));
    // The directory containing the manifest (cargo.toml) of this package
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Couldn't get CARGO_MANIFEST_DIR envVar"),
    );

    // Since we already fenced this build script to run only if changes to shaders are made, we can always run this
    cross_compile_glsl_shaders();

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

fn cross_compile_glsl_shaders() {
    // Create our shader cross-compiler
    let mut compiler = shaderc::Compiler::new().expect("Could not create GLSL -> SPIR-V compiler");
    let options =
        shaderc::CompileOptions::new().expect("Could not create GLSL -> SPIR-V compiler options"); // Can alter compiler options here

    // Create a glsl->spirv destination path if neccessary
    fs::create_dir_all("assets/shaders/spirv").expect("Couldn't create SPIR-V output dir");
    // Create a spirv->hlsl destination path if neccessary
    fs::create_dir_all("assets/shaders/hlsl").expect("Couldn't create HLSL output dir");
    // Create a spirv->msl destination path if neccessary
    fs::create_dir_all("assets/shaders/metal").expect("Couldn't create Metal output dir");

    // Loop over all glsl shaders to cross-compile them to spir-v format
    for entry in fs::read_dir("assets/shaders/glsl").expect("Cannot read dir: assets/shaders/glsl")
    {
        let entry: fs::DirEntry = entry.expect("Couldn't grab DirEntry");
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
                        // TODO: Others?
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
                        // GLSL -> SPIR-V succeeded, write the output to a SPIR-V file
                        let num_warnings = compiled_spirv.get_num_warnings();
                        let warning_msgs = compiled_spirv.get_warning_messages();
                        println!(
                            "{} GLSL -> SPIR-V cross-compilation succeeded with {} warnings:\n{}",
                            filename, num_warnings, warning_msgs
                        );
                        let compiled_bytes = compiled_spirv.as_binary_u8();
                        let out_path = format!("assets/shaders/spirv/{}.spv", filename);
                        fs::write(&out_path, &compiled_bytes)
                            .expect("Couldn't write compiled SPIR-V shader to output dir");
                        // Now SPIR-V -> HLSL + MSL
                        let spirv_module = spirv::Module::from_words(compiled_spirv.as_binary());
                        create_hlsl_from_compiled_spirv(&filename, &spirv_module);
                        create_msl_from_compiled_spirv(&filename, &spirv_module);
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

/// Creates an equivalent .hlsl (DirectX) shader file from a compiled SPIR-V shader
fn create_hlsl_from_compiled_spirv(filename: &str, spirv_module: &spirv::Module) {
    let mut abstract_syntax_tree = spirv::Ast::<hlsl::Target>::parse(&spirv_module)
        .expect("Couldn't parse abstract syntax tree (HLSL target) from SPIR-V module");
    let hlsl_output = abstract_syntax_tree
        .compile()
        .expect("Couldn't compile SPIR-V abstract syntax tree to HLSL");
    use std::fs::File;
    use std::io::prelude::*;
    let mut hlsl_file_out = File::create(format!("assets/shaders/hlsl/{}.hlsl", filename))
        .expect("Couldn't create new HLSL file");
    hlsl_file_out.write_all(hlsl_output.as_bytes());
}

/// Creates an equivalent .metal (macOS) shader file from a compiled SPIR-V shader
fn create_msl_from_compiled_spirv(filename: &str, spirv_module: &spirv::Module) {
    let mut abstract_syntax_tree = spirv::Ast::<msl::Target>::parse(&spirv_module)
        .expect("Couldn't parse abstract syntax tree (Metal target) from SPIR-V module");
    let msl_output = abstract_syntax_tree
        .compile()
        .expect("Couldn't compile SPIR-V abstract syntax tree to Metal");
    use std::fs::File;
    use std::io::prelude::*;
    let mut msl_file_out = File::create(format!("assets/shaders/metal/{}.metal", filename))
        .expect("Couldn't create new Metal file");
    msl_file_out.write_all(msl_output.as_bytes());
}
