fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(false)
        .compile_protos(&["proto/tron/api/api.proto"], &["proto/tron", "proto"])?;
    Ok(())
}
