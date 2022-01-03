fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]\n#[serde(default)]")
        .compile(
            &[
                "proto/proto_example.proto",
                "proto/proto_dendrite_config.proto"
            ],
            &[
                "proto"
            ])?;
    Ok(())
}