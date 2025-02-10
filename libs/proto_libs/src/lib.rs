pub mod post_proto {
    tonic::include_proto!("post");
}

pub const POST_FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("post_descriptor");