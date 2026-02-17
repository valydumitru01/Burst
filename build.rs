use std::fs;
use std::path::Path;
use shaderc::{Compiler, ShaderKind, CompileOptions};

fn main() {
    println!("cargo:rerun-if-changed=gapi/rendering/");

    let mut compiler = Compiler::new().expect("Failed to create shader compiler");
    let mut options = CompileOptions::new().unwrap();
    options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let vert_src = root.join("src/gapi/shaders/shader.vert");
    let frag_src = root.join("src/gapi/shaders/shader.frag");

    // Just the filenames, not the full paths yet
    let shaders = [
        (vert_src.to_str().unwrap(), "vert.spv", ShaderKind::Vertex),
        (frag_src.to_str().unwrap(), "frag.spv", ShaderKind::Fragment),
    ];

    let out_dir = std::env::var("OUT_DIR").unwrap();

    for (source_path, file_name, kind) in shaders {
        let source_code = fs::read_to_string(source_path)
            .expect(&format!("Failed to read shader: {}", source_path));

        let binary_result = compiler.compile_into_spirv(
            &source_code,
            kind,
            source_path,
            "main",
            Some(&options),
        ).expect(&format!("Failed to compile shader: {}", source_path));

        // Construct the full path here
        let dest_path = Path::new(&out_dir).join(file_name);

        fs::write(&dest_path, binary_result.as_binary_u8())
            .expect("Failed to write SPIR-V file");
    }
}