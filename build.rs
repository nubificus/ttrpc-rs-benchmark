use std::env;
use std::path::PathBuf;

fn main() {
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src");
    let proto_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    ttrpc_codegen::Codegen::new()
        .out_dir(&src_dir)  // Generate directly to src/ directory
        .inputs(&[proto_path.join("echo.proto")])
        .include(&proto_path)
        .rust_protobuf()
        .run()
        .expect("Generate code failed.");
}
