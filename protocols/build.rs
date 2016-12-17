#[macro_use]
extern crate combustion_common as common;
extern crate capnpc;
extern crate cmake;
extern crate gcc;

use std::process::Command;
use std::env;
use std::fs::*;
use std::path::Path;

use common::error::*;
use common::utils;

/// Visit directories, find .capnp files, compile them, then replace absolute module references with `super` in the output code.
fn compile_capnprotos() {
    info!("Compiling Cap'N Proto protocols");

    utils::fs::visit_dirs(Path::new("protocols"), &|entry: &DirEntry| {
        if let Some(ext) = entry.path().as_path().extension() {
            if ext == "capnp" {
                info!("Attempting to generate: {:?} as Rust", entry.path());

                capnpc::CompilerCommand::new().file(entry.path()).include("protocols").run().expect_logged("Failed to compile protocol");

                #[cfg(feature = "cpp")]
                {
                    info!("Attempting to generate: {:?} as C++", entry.path());
                    let output = Command::new("capnp.exe").arg("compile").arg("-oc++").arg("-Iprotocols").arg(entry.path()).output()
                                                          .expect_logged("Failed to compile protocol");

                    if !output.status.success() {
                        error!("Output: {:?}", output);
                        panic!("Output: {:?}", output);
                    }
                }

                info!("Success!");
            }
        }
    });

    #[cfg(feature = "cpp")]
    {
        info!("Moving generated C++ code to cpp directory");

        Command::new("mv")
            .arg("protocols/*.capnp.h")
            .arg("protocols/*.capnp.c++")
            .arg("protocols/*.capnp.cpp")
            .arg("cpp")
            .output()
            .expect_logged("Could not move files");

        info!("Success!");
    }

    info!("Finished all Cap'N Proto protocols");
}

#[cfg(feature = "cpp")]
fn build_cpp() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let src_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    info!("Compiling Cap'N Proto C++ static library");

    let mut capnp_config = cmake::Config::new("cpp/capnproto/c++");

    capnp_config.define("CAPNP_LITE", "1");
    capnp_config.define("EXTERNAL_CAPNP", "1");
    capnp_config.define("BUILD_TOOLS", "OFF");
    capnp_config.define("BUILD_TESTING", "OFF");

    capnp_config.define("CAPNP_EXECUTABLE", format!("{}/capnp.exe", src_dir));
    capnp_config.define("CAPNP_CXX_EXECUTABLE", format!("{}/capnpc-c++.exe", src_dir));
    capnp_config.define("CAPNP_INCLUDE_DIRECTORY", format!("{}/cpp/capnproto/c++/src/capnp/", src_dir));

    let capnp_dst = capnp_config.build();

    println!("cargo:rustc-link-search=native={}", capnp_dst.join("lib").display());
    println!("cargo:rustc-link-lib=static=capnp");
    println!("cargo:rustc-link-lib=static=kj");

    Command::new("cp").arg("-R")
                      .arg(format!("{}/include/*", out_dir))
                      .arg("cpp/include/")
                      .output()
                      .expect_logged("Failed to copy include files");

    info!("Success!");
}

fn main() {
    common::log::init_global_logger("logs/build").unwrap();

    compile_capnprotos();

    #[cfg(feature = "cpp")]
    build_cpp();
}
