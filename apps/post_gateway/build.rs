use std::{
    error::Error,
    env::var,
    path::PathBuf
};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());

    tonic_build::configure()
    .file_descriptor_set_path(out_dir.join("post_descriptor.bin"))
    .compile_protos(&["proto/post.proto"], &["proto"])?;

    tonic_build::compile_protos("proto/post.proto")?;

    Ok(())
}
