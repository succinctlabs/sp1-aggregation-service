fn main() {
    let config = tonic_build::configure();
    config
        .protoc_arg("--experimental_allow_proto3_optional")
        .out_dir("src")
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .type_attribute(".aggregation.ProofRequest", "#[derive(sqlx::FromRow)]")
        .type_attribute(".aggregation.AggregationStatus", "#[derive(sqlx::Type)]")
        .type_attribute(".aggregation.ResponseStatus", "#[derive(sqlx::Type)]")
        .compile_protos(&["../proto/aggregation.proto"], &["../proto"])
        .unwrap();
}
