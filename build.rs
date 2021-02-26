fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/grpc_example.proto")?;
    tonic_build::compile_protos("proto/dendrite_config.proto")?;
    Ok(())
}