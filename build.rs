use cc;
use std::path::PathBuf;

fn main() {
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
        pb.push("..");
        pb.push("afsktest");
        pb.push("src");
        pb.push(f);
        c_paths.push(pb);
    }

    cc::Build::new()
        .files(c_paths)
        .define("NO_MAIN", Some("bar"))
        .include("../afsktest/include")
        .compile("afsktest");
}
