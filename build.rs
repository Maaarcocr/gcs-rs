fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "googleapis";
    println!("cargo:rerun-if-changed={}", proto_root);
    protoc_grpcio::compile_grpc_protos(
        &[
            "google/storage/v2/storage.proto",
            "google/type/date.proto",
            "google/iam/v1/policy.proto",
            "google/iam/v1/iam_policy.proto",
            "google/iam/v1/options.proto",
            "google/type/expr.proto",
        ],
        &[proto_root],
        &"src/protos",
        None,
    )
    .expect("Failed to compile gRPC definitions!");
    Ok(())
}
