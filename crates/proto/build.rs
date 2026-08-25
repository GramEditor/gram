fn main() {
    println!("cargo:rerun-if-changed=proto");
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }
    let mut build = prost_build::Config::new();
    match build
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&["proto/gram.proto"], &["proto"])
    {
        Ok(()) => (),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
