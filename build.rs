use cc;
use std::path::PathBuf;

fn main() {
    let h_files = &["esplanade_demod.h", "esplanade_mac.h", "murmur3.h"];
    for f in h_files {
        println!("cargo:rerun-if-changed=afsk-core/include/{}", f);
    }

    let c_files = &[
        "dsptable_gen.c",
        "esplanade_demod.c",
        "esplanade_mac.c",
        "main.c",
        "murmur3.c",
    ];

    let mut c_paths = vec![];
    for f in c_files {
        let mut pb = PathBuf::new();
        pb.push("afsk-core");
        pb.push("src");
        pb.push(f);
        println!("cargo:rerun-if-changed=afsk-core/src/{}", f);
        c_paths.push(pb);
    }

    cc::Build::new()
        .files(c_paths)
        .define("NO_MAIN", None)
        .include("afsk-core/include")
        .warnings_into_errors(true)
        .debug(true)
        .compile("afsktest");
}
