use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

use error::*;

#[derive(Debug, Clone)]
pub struct ShaderResource {
    pub path_vert : Option<PathBuf>,
    pub path_frag : Option<PathBuf>
}

pub fn read_file_to_string(filename : &str) -> Result<String> {
    let mut file = File::open(&filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn write_file(out_file : &str, contents : &str) -> Result<()> {
    let mut file = File::create(out_file)?;
    file.write(contents.as_bytes())?;
    Ok(())
}

pub fn gather_shader_files_in_directory(path : &str) -> Result<Vec<ShaderResource>> {
    let mut files : HashMap<_, ShaderResource> = HashMap::new();
    let mut ret = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            ret.append(&mut gather_shader_files_in_directory(path.to_str().unwrap())?);
        } else if path.is_file() {
            if let Some(stem) = path.file_stem() {
                let o_entry = if let Some(entry) = files.get_mut(stem) {
                    entry.clone()
                } else {
                    ShaderResource {
                        path_vert: None,
                        path_frag: None
                    }
                };
                let n_entry = match path.extension().map(|a| a.to_str()) {
                    Some(ext) => match ext {
                        Some("vert") => Some(ShaderResource {
                            path_vert: Some(path.to_owned()),
                            ..o_entry
                        }),
                        Some("frag") => Some(ShaderResource {
                            path_frag: Some(path.to_owned()),
                            ..o_entry
                        }),
                        _ => None
                    },
                    None => None
                };
                if let Some(n_entry) = n_entry {
                    files.insert(stem.to_owned(), n_entry);
                }
            }
        }
    }
    ret.extend(files.values().map(|a| a.clone()));
    Ok(ret)
}


