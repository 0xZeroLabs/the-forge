fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &[
            "external/eigenda/api/proto/disperser/disperser.proto",
            "external/eigenda/api/proto/common/common.proto",
        ],
        &["external/eigenda/api/proto"],
    )?;
    Ok(())
}
