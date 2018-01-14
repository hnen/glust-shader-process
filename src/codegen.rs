
use glust::AttribInfo;
use glust::UniformInfo;
//use rustfmt;
use quote::Tokens;
use syn;
use gl;
use gl::types::GLenum;

use filesystem;

use error::*;


pub fn generate_rust_module(vert_path : &str, frag_path : &str, uniforms : &[UniformInfo], attribs_sorted : &[AttribInfo]) -> Result<Tokens> {

    let uniforms_code = uniforms_code(uniforms)?;
    let attribs = attribs_code(attribs_sorted)?;
    let attribs_tmp = attribs_tmp_code(attribs_sorted)?;
    let vertex_array_fields = va_fields_code(attribs_sorted, false)?;
    let vertex_array_fields_ref = va_fields_code(attribs_sorted, true)?;
    //let uniform_values = uniform_values_code(uniforms, "uniforms")?;
    let uniform_values_self = uniform_values_code(uniforms, "self")?;

    let vs_code = filesystem::read_file_to_string(vert_path)?;
    let fs_code = filesystem::read_file_to_string(frag_path)?;

    Ok(quote! {
        use glust::GlBuffer;
        use glust::GlShader;
        use glust::HasGlVertexArrayHandle;
        use glust::GlVertexArray;
        use glust::GlVertexArrayTmp;
        use glust::GlShaderUniform;
        use glust::GlError;

        use renderer_glust::ShaderUniforms;
        //use renderer_glust::ShaderVertexArray;

        type Result<T> = ::std::result::Result<T, GlError>;

        pub static VS_PATH : &'static str = #vert_path;
        pub static FS_PATH : &'static str = #frag_path;

        static VS_CODE : &'static str = #vs_code;
        static FS_CODE : &'static str = #fs_code;

        pub struct Uniforms {
            #uniforms_code
        }

        pub struct Attribs {
            #attribs
        }
        pub struct AttribsTmp<'a> {
            #attribs_tmp
        }

        pub struct Shader(pub GlShader);
        pub struct VertexArray(pub GlVertexArray);
        pub struct VertexArrayTmp<'a>(pub GlVertexArrayTmp<'a>);

        impl VertexArray {
            pub fn new(attribs : Attribs) -> Result<VertexArray> {
                Ok(VertexArray(GlVertexArray::new(vec![ #vertex_array_fields ])?))
            }
        }
        impl<'a> VertexArrayTmp<'a> {
            pub fn new(attribs : AttribsTmp<'a>) -> Result<VertexArrayTmp<'a>> {
                Ok(VertexArrayTmp(GlVertexArrayTmp::new(vec![ #vertex_array_fields_ref ])?))
            }
        }
        impl ::renderer_glust::ShaderVertexArray for VertexArray {
            fn gl_vertex_array<'a>(&'a self) -> &'a GlVertexArray {
                &self.0
            }
        }

        impl ::renderer_glust::OfShader<Shader> for VertexArray {}
        impl<'a> ::renderer_glust::OfShader<Shader> for VertexArrayTmp<'a> {}

        impl HasGlVertexArrayHandle for VertexArray {
            fn gl_vao_handle(&self) -> u32 {
                self.0.gl_vao_handle()
            }
        }
        impl<'a> HasGlVertexArrayHandle for VertexArrayTmp<'a> {
            fn gl_vao_handle(&self) -> u32 {
                self.0.gl_vao_handle()
            }
        }

        impl ::renderer_glust::Shader for Shader {
            type VertexArray = VertexArray;
            type Uniforms = Uniforms;

            fn gl_shader<'a>(&'a self) -> &'a GlShader {
                &self.0
            }
        }

        impl ShaderUniforms for Uniforms {
            fn uniform_array(&self) -> Vec<(&str, GlShaderUniform)> {
                vec![ #uniform_values_self ]
            }
        }


        impl Shader {
            pub fn compile() -> Result<Self> {
                Ok(Shader(GlShader::compile(VS_CODE, FS_CODE)?))
            }
        }
    })
}

fn uniform_values_code(uniforms : &[UniformInfo], uniform_struct_id : &str) -> Result<Tokens> {
    let uniform_struct = syn::Ident::from(uniform_struct_id);
    let mut values = Vec::new();
    for uniform in uniforms {
        let name_str = &uniform.name;
        let name_id = syn::Ident::from(uniform.name.clone());
        let glust_enum = glsl_type_to_glust_uniform_enum(uniform.datatype)?;
        let elem = quote!{ (#name_str, #glust_enum(#uniform_struct.#name_id) ) };
        values.push(elem);
    }

    Ok(quote!{ #(#values),* })

}

fn uniforms_code(uniforms : &[UniformInfo]) -> Result<Tokens> {
    let mut code = quote! {};
    for uniform in uniforms {
        let field_code = rust_field(&uniform.name, uniform.datatype, uniform.size)?;
        code = quote!{ #code #field_code, }
    }
    Ok(code)
}

fn va_fields_code(attribs : &[AttribInfo], as_ref : bool) -> Result<Tokens> {
    let mut fields = Vec::new();
    for attrib in attribs {
        let name = syn::Ident::from(attrib.name.clone());
        if !as_ref {
            fields.push(quote! { attribs.#name.0 })
        } else {
            fields.push(quote! { &attribs.#name.0 })
        }
    }
    Ok(quote!{ #(#fields),* })
}


fn attribs_code(attribs : &[AttribInfo]) -> Result<Tokens> {
    let mut code = quote! {};
    for attrib in attribs {
        let field_code = rust_buffer_field(&attrib.name, attrib.datatype, attrib.size, false)?;
        code = quote!{ #code #field_code, }
    }
    Ok(code)
}

fn attribs_tmp_code(attribs : &[AttribInfo]) -> Result<Tokens> {
    let mut code = quote! {};
    for attrib in attribs {
        let field_code = rust_buffer_field(&attrib.name, attrib.datatype, attrib.size, true)?;
        code = quote!{ #code #field_code, }
    }
    Ok(code)
}

fn rust_buffer_field(name : &String, datatype : GLenum, size : i32, as_ref : bool) -> Result<Tokens> {
    let rtype = glsl_type_to_rust(datatype)?;
    let name_el = syn::Ident::from(name.clone());
    match size {
        1 => {
            if !as_ref {
                Ok(quote! { pub #name_el : GlBuffer<#rtype> })
            } else {
                Ok(quote! { pub #name_el : &'a GlBuffer<#rtype> })
            }
        },
        _ => {
            bail!("Unsupported field size of {} for {}", size, name);
        }
    }
}

fn rust_field(name : &String, datatype : GLenum, size : i32) -> Result<Tokens> {
    let rtype = glsl_type_to_rust(datatype)?;
    let name_el = syn::Ident::from(name.clone());
    match size {
        1 => {
            Ok(quote!{ pub #name_el : #rtype })
        },
        _ => {
            bail!("Unsupported field size of {} for {}", size, name);
        }
    }
}


fn glsl_type_to_glust_uniform_enum(type_enum : GLenum) -> Result<Tokens> {
    match type_enum {
        gl::FLOAT => Ok(quote!( GlShaderUniform::Float )),
        gl::FLOAT_VEC2 => Ok(quote!( GlShaderUniform::Vec2 )),
        gl::FLOAT_VEC3 => Ok(quote!( GlShaderUniform::Vec3 )),
        gl::FLOAT_VEC4 => Ok(quote!( GlShaderUniform::Vec4 )),
        gl::SAMPLER_2D => Ok(quote!( GlShaderUniform::TextureHandle )),
        gl::FLOAT_MAT4 => Ok(quote!( GlShaderUniform::Mat4x4 )),
        gl::INT => Ok(quote!( GlShaderUniform::Int )),
        x => Err(format!("Unsupported GLSL type: {:?}", x).into())
    }
}

fn glsl_type_to_rust(type_enum : GLenum) -> Result<Tokens> {
    match type_enum {
        gl::FLOAT => Ok(quote!( f32 )),
        gl::FLOAT_VEC2 => Ok(quote!( [f32;2] )),
        gl::FLOAT_VEC3 => Ok(quote!( [f32;3] )),
        gl::FLOAT_VEC4 => Ok(quote!( [f32;4] )),
        gl::SAMPLER_2D => Ok(quote!( u32 )),
        gl::FLOAT_MAT4 => Ok(quote!( [f32;16] )),
        gl::INT => Ok(quote!( i32 )),
        x => Err(format!("Unsupported GLSL type: {:?}", x).into())
    }
}
