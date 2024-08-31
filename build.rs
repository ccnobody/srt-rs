use std::env;
use std::path::PathBuf;
use std::fs;
use std::process::Command;

fn mkdir()->(PathBuf,PathBuf) {
    // 设置SRT仓库的目标路径和安装路径
    let srt_source_path = PathBuf::from("depends/build/srt");
    let srt_install_path = PathBuf::from("depends/srt");
    // 确保目录存在
    fs::create_dir_all(&srt_source_path).expect("Failed to create SRT source directory");
    fs::create_dir_all(&srt_install_path).expect("Failed to create SRT install directory");

    println!("SRT source path: {}", srt_source_path.display());
    println!("SRT install path: {}", srt_install_path.display());

    (srt_source_path.canonicalize().unwrap(),srt_install_path.canonicalize().unwrap())
}

fn compile_srt_lib(){

    let  (srt_source_path, srt_install_path) = mkdir();
    // 克隆SRT仓库
    if srt_source_path.read_dir().map(|mut i| i.next().is_none()).unwrap_or(true) {
        println!("Cloning SRT repository...");
        Command::new("git")
            .current_dir(&srt_source_path.parent().unwrap())
            .args(&["clone", "https://github.com/Haivision/srt.git", "srt"])
            .status()
            .expect("无法克隆SRT仓库");
    } else {
        println!("SRT repository already exists, skipping clone.");
    }

    // 编译并安装SRT
    let srt_build_dir = srt_source_path.join("build");
    std::fs::create_dir_all(&srt_build_dir).expect("无法创建SRT构建目录");

    // 在 CMake 配置中使用 pkg-config 的输出
    Command::new("cmake")
        .current_dir(&srt_build_dir)
        .args(&[
            "..",
            &format!("-DCMAKE_INSTALL_PREFIX={}", srt_install_path.to_str().unwrap()),
            "-DENABLE_SHARED=OFF",
            "-DENABLE_STATIC=ON",
            "-DUSE_STATIC_LIBSTDCXX=ON",
            "-DUSE_ENCLIB=openssl",
            "-DENABLE_CXX11=ON"
        ])
        .status()
        .expect("CMake配置失败");

    Command::new("cmake")
        .current_dir(&srt_build_dir)
        .args(&["--build", ".", "--config", "Release"])
        .status()
        .expect("CMake构建失败");

    Command::new("cmake")
        .current_dir(&srt_build_dir)
        .args(&["--install", "."])
        .status()
        .expect("CMake安装失败");
}

fn main() {
    let  (_, srt_install_path) = mkdir();
    compile_srt_lib();

    // 设置链接路径
    println!("cargo:rustc-link-search=native={}/lib", srt_install_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=srt");

    // 告诉cargo在SRT源码或wrapper.h发生变化时重新运行此脚本
    println!("cargo:rerun-if-changed=depends/build/srt");
    println!("cargo:rerun-if-changed=wrapper.h");

    // SRT头文件的路径
    let srt_include_path = srt_install_path.join("include");

    // 使用bindgen生成绑定
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", srt_include_path.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("无法生成绑定");

    // 将生成的绑定写入文件
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("无法写入绑定");
}
