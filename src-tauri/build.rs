// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use tauri_build::Attributes;

fn main() {
    // 通过 Attributes 设置精确的 capabilities 路径（指向具体文件而非目录），
    // 避免 tauri-build 默认 emit `cargo:rerun-if-changed=capabilities`（目录），
    // 目录 mtime 变化会导致 Cargo 不必要地重新编译。
    let attrs = Attributes::new().capabilities_path_pattern("./capabilities/**/*");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=capabilities/default.json");

    tauri_build::try_build(attrs).unwrap_or_else(|e| {
        println!("{e:#}");
        std::process::exit(1);
    });
}
