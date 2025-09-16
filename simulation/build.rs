use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // While OUT_DIR is set for both build.rs and compiling the crate, PROFILE is only set in
    // build.rs. So, export it to crate compilation as well.
    let profile = env::var("PROFILE").unwrap();
    println!("cargo:rustc-env=PROFILE={profile}");

    build_shader("../shaders/mouse-shader", true)?;
    Ok(())
}

fn build_shader(path_to_crate: &str, codegen_names: bool) -> Result<(), Box<dyn Error>> {
    let builder_dir = &Path::new(env!("CARGO_MANIFEST_DIR"));
    let path_to_crate = builder_dir.join(path_to_crate);
    let result = cargo_gpu::Install::from_shader_crate(path_to_crate.clone())
        .run()
        .unwrap()
        .to_spirv_builder(path_to_crate, "spirv-unknown-vulkan1.1")
        .print_metadata(cargo_gpu::spirv_builder::MetadataPrintout::Full)
        .build()?;

    if codegen_names {
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("entry_points.rs");
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(dest_path, result.codegen_entry_point_strings()).unwrap();
    }
    Ok(())
}
