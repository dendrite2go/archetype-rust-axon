use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("OUT_DIR", "src");
    tonic_build::compile_protos("proto/proto_example.proto")?;
    tonic_build::compile_protos("proto/proto_dendrite_config.proto")?;
    Ok(())
}