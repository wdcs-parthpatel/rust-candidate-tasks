fn main() {
    protobuf_codegen::Codegen::new()
        .out_dir("src/generated/")
        .inputs(&["proto/test.proto"])
        .include("proto")
        .run()
        .expect("Failed to run Protobuf codegen");
}
