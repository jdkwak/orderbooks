fn main() {
    println!("cargo:rerun-if-changed=proto/orderbook.proto");
    tonic_build::configure()
        .compile_protos(&["proto/orderbook.proto"], &["proto"])
        .expect("Failed to compile proto");
}
