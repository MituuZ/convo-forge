/*
 * Copyright © 2025 Mitja Leino
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
 * documentation files (the “Software”), to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
 * and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE
 * WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS
 * OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=src/user_tools");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("user_tools_gen.rs");

    let user_tools_dir = std::path::Path::new("src/user_tools");
    let mut modules = vec![];

    if user_tools_dir.exists() {
        for entry in fs::read_dir(user_tools_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") &&
                path.file_stem().and_then(|stem| stem.to_str()) != Some("mod") {
                let file_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
                modules.push(file_stem.clone());

                let dest_file = std::path::Path::new(&out_dir).join(format!("{}.rs", file_stem));
                fs::copy(&path, &dest_file).unwrap();
            }
        }
    }

    let mut code = String::new();
    for m in &modules {
        code.push_str(&format!("pub mod {};\n", m));
        code.push_str(&format!("use {m}::tool as {m}_tool;\n"))
    }

    code.push_str("use crate::tool::tools::Tool;\n");
    code.push_str(
        "pub fn get_user_tools() -> Vec<Tool> {\n\
            let mut v = Vec::new();\n\
            ");

    for m in &modules {
        code.push_str(&format!("v.push({m}_tool());\n", m = m));
    }

    code.push_str("\n    v\n}\n");

    fs::write(&dest_path, code).unwrap();
}