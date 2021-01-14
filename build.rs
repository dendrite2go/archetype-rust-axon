fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/hello_world.proto")?;
    tonic_build::compile_protos("proto/grpc_example.proto")?;
    tonic_build::configure().build_server(false).compile(
        &[
            "proto/axon_server/command.proto",
            "proto/axon_server/control.proto",
            "proto/axon_server/event.proto",
            "proto/axon_server/query.proto"
        ],
        &["proto/axon_server"]
    )?;
    Ok(())
}