#![recursion_limit="1024"]

#[macro_use] extern crate quote;
#[macro_use] extern crate error_chain;
extern crate glust;
extern crate glutin;
extern crate gl;
extern crate rustfmt;
extern crate syn;

use glutin::GlContext;

mod filesystem;
mod error;
mod codegen;

use error::*;
use filesystem::ShaderResource;

use glust::GlShader;
use std::path::PathBuf;

pub fn process_directory(path : &str) -> Result<()> {

    let _ctx = create_gl_context()?;

    let files = filesystem::gather_shader_files_in_directory(path)?;

    println!("{:#?}", files);

    for file in files {
        process_shader(&file)?;
    }

    //unimplemented!();

    Ok(())
}

fn process_shader(res : &ShaderResource) -> Result<()> {
    let (ref vs_path, ref fs_path) = match (&res.path_vert, &res.path_frag) {
        (&Some(ref vs_path), &Some(ref fs_path)) => {
            (vs_path, fs_path)
        },
        (&Some(ref vs_path), &None) => {
            Err(format!("Missing fragment shader for {:?}", vs_path))?
        },
        (&None, &Some(ref fs_path)) => {
            Err(format!("Missing vertex shader for {:?}", fs_path))?
        },
        _ => {
            Err("Something went wrong.")?
        }
    };

    let vs_path_str = vs_path.to_str().ok_or("Couldn't encode path")?;
    let fs_path_str = fs_path.to_str().ok_or("Couldn't encode path")?;

    let vs_src = filesystem::read_file_to_string(vs_path_str)?;
    let fs_src = filesystem::read_file_to_string(fs_path_str)?;

    let shader = GlShader::compile(&vs_src[..], &fs_src[..])?;

    let uniforms = shader.get_uniform_infos()?;
    let attribs = shader.get_attrib_infos_sorted()?;

    let module = codegen::generate_rust_module(vs_path_str, fs_path_str, &uniforms[..], &attribs[..])?;

    let out_file = format!("{}/{}.rs", vs_path.parent().ok()?.to_str().ok()?, vs_path.file_stem().ok()?.to_str().ok()?);

    println!("out_file: {}", out_file);

    filesystem::write_file(out_file.as_str(), module.to_string().as_str())?;

    let mut conf = rustfmt::config::Config::default();
    conf.set().max_width(10240);
    conf.set().fn_empty_single_line(true);

    let out_file_backup = format!("{}.bk", out_file);

    let summary = rustfmt::run(rustfmt::Input::File(PathBuf::from(out_file)), &conf);
    ::std::fs::remove_file(PathBuf::from(out_file_backup)).unwrap();

    if !summary.has_no_errors() {
        bail!("Rustfmt failed: {:?}", summary);
    }

    Ok(())
}

fn create_gl_context() -> Result<glutin::HeadlessContext> {
    let gl_ctx = glutin::HeadlessRendererBuilder::new(1, 1).with_gl_debug_flag(true).build()?;
    unsafe { gl_ctx.make_current()? };
    gl::load_with(|symbol| gl_ctx.get_proc_address(symbol) as *const _);
    Ok(gl_ctx)
}