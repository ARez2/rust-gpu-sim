use clap::Parser;
use clap::ValueEnum;
use std::borrow::Cow;
use std::path::PathBuf;
use strum::{Display, EnumString};
use winit::event_loop::EventLoop;

use crate::application::Application;

// NOTE(eddyb) while this could theoretically work on the web, it needs more work.
// #[cfg(not(target_arch = "wasm32"))]
// mod compute;

mod application;
mod simulation;

#[derive(Debug, EnumString, Display, PartialEq, Eq, Copy, Clone, ValueEnum)]
pub enum RustGPUShader {
    Simplest,
    Sky,
    Compute,
    Mouse,
}

fn compile_and_watch(
    shader_crate_name: &str,
    spirv_passthrough: bool,
    #[cfg(not(target_arch = "wasm32"))] on_watch: Option<
        Box<dyn FnMut(wgpu::ShaderModuleDescriptorSpirV<'static>) + Send + 'static>,
    >,
) -> wgpu::ShaderModuleDescriptorSpirV<'static> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::path::PathBuf;

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let crate_path = [manifest_dir, "..", "shaders", shader_crate_name]
            .iter()
            .copied()
            .collect::<PathBuf>();

        let has_debug_printf = spirv_passthrough;

        let builder = cargo_gpu::Install::from_shader_crate(crate_path.clone())
            .run()
            .unwrap()
            .to_spirv_builder(crate_path, "spirv-unknown-vulkan1.1");
        let builder = builder
            .print_metadata(cargo_gpu::spirv_builder::MetadataPrintout::None)
            .shader_panic_strategy(if has_debug_printf {
                cargo_gpu::spirv_builder::ShaderPanicStrategy::DebugPrintfThenExit {
                    print_inputs: true,
                    print_backtrace: true,
                }
            } else {
                cargo_gpu::spirv_builder::ShaderPanicStrategy::SilentExit
            });

        fn handle_compile_result(
            compile_result: cargo_gpu::spirv_builder::CompileResult,
        ) -> wgpu::ShaderModuleDescriptorSpirV<'static> {
            match compile_result.module {
                cargo_gpu::spirv_builder::ModuleResult::SingleModule(path) => {
                    load_spirv_module(path)
                }
                cargo_gpu::spirv_builder::ModuleResult::MultiModule(_modules) => unreachable!(),
                // modules
                //     .into_iter()
                //     .map(|(name, path)| load_spirv_module(path))
                //     .collect()
            }
        }

        if let Some(mut f) = on_watch {
            builder
                .watch(move |compile_result, accept| {
                    let modules = handle_compile_result(compile_result);
                    if let Some(accept) = accept {
                        accept.submit(modules);
                    } else {
                        f(modules);
                    }
                })
                .expect("Configuration is correct for watching")
                .unwrap()
        } else {
            handle_compile_result(builder.build().unwrap())
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        // let module = match options.shader {
        //     RustGPUShader::Simplest => {
        //         wgpu::include_spirv_raw!(env!("simplest_shader.spv"))
        //     }
        //     RustGPUShader::Sky => wgpu::include_spirv_raw!(env!("sky_shader.spv")),
        //     RustGPUShader::Compute => wgpu::include_spirv_raw!(env!("compute_shader.spv")),
        //     RustGPUShader::Mouse => wgpu::include_spirv_raw!(env!("mouse_shader.spv")),
        // };
        let module = wgpu::include_spirv_raw!(env!("mouse_shader.spv"));
        let spirv = match module {
            wgpu::ShaderModuleDescriptorPassthrough::SpirV(spirv) => spirv,
            _ => panic!("not spirv"),
        };
        CompiledShaderModules {
            named_spv_modules: vec![(None, spirv)],
        }
    }
}

fn load_spirv_module(path: PathBuf) -> wgpu::ShaderModuleDescriptorSpirV<'static> {
    let data = std::fs::read(path).unwrap();
    // FIXME(eddyb) this reallocates all the data pointlessly, there is
    // not a good reason to use `ShaderModuleDescriptorSpirV` specifically.
    let spirv = Cow::Owned(wgpu::util::make_spirv_raw(&data).into_owned());
    wgpu::ShaderModuleDescriptorSpirV {
        label: None,
        source: spirv,
    }
}

#[derive(Parser, Clone)]
#[command()]
pub struct Options {
    /// which shader to run
    #[cfg_attr(not(target_arch = "wasm32"), arg(short, long, default_value = "sky"))]
    #[cfg_attr(target_arch = "wasm32", arg(short, long, default_value = "mouse"))]
    shader: RustGPUShader,

    #[arg(long)]
    force_spirv_passthru: bool,

    #[structopt(long)]
    emulate_push_constants_with_storage_buffer: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = Options::parse();
    let event_loop = EventLoop::new()?;
    let mut app = Application::default();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Hack: spirv_builder builds into a custom directory if running under cargo, to not
        // deadlock, and the default target directory if not. However, packages like `proc-macro2`
        // have different configurations when being built here vs. when building
        // rustc_codegen_spirv normally, so we *want* to build into a separate target directory, to
        // not have to rebuild half the crate graph every time we run. So, pretend we're running
        // under cargo by setting these environment variables.
        unsafe {
            std::env::set_var("OUT_DIR", env!("OUT_DIR"));
            std::env::set_var("PROFILE", env!("PROFILE"));
        }

        // if options.shader == RustGPUShader::Compute {
        //     return compute::start(&options);
        // }
    }

    // HACK(eddyb) force push constant emulation using (read-only) SSBOs, on
    // wasm->WebGPU, as push constants are currently not supported.
    // FIXME(eddyb) could push constant support be automatically detected at runtime?
    if cfg!(target_arch = "wasm32") {
        options.emulate_push_constants_with_storage_buffer = true;
    }

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(&mut app).map_err(Into::into)
}
