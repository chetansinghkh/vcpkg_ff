// use std::env;
// use std::path::PathBuf;

fn main() {
    // // 设置vcpkg根目录（在运行目录下）
    // let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // let vcpkg_root = PathBuf::from(&manifest_dir).join("vcpkg");
    
    // println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=vcpkg");
    
    // // 设置vcpkg环境变量
    // env::set_var("VCPKG_ROOT", vcpkg_root.to_str().unwrap());
    
    // // 尝试使用vcpkg查找ffmpeg和x264
    // // 注意：如果vcpkg还未安装，这里会失败，但不会阻止编译
    // // 用户需要先运行程序来安装vcpkg和包
    // if let Ok(mut vcpkg) = vcpkg::Config::new() {
    //     // 设置triplet为x64-windows-static
    //     vcpkg.target_triplet("x64-windows-static");
        
    //     // 查找ffmpeg
    //     if let Ok(lib) = vcpkg.probe_package("ffmpeg") {
    //         for path in &lib.link_paths {
    //             println!("cargo:rustc-link-search=native={}", path.display());
    //         }
    //         println!("cargo:rustc-link-lib=static=avcodec");
    //         println!("cargo:rustc-link-lib=static=avformat");
    //         println!("cargo:rustc-link-lib=static=avutil");
    //         println!("cargo:rustc-link-lib=static=swscale");
    //         println!("cargo:rustc-link-lib=static=swresample");
    //     } else {
    //         println!("cargo:warning=ffmpeg未找到，请先运行程序安装vcpkg和ffmpeg");
    //     }
        
    //     // 查找x264
    //     if let Ok(lib) = vcpkg.probe_package("x264") {
    //         for path in &lib.link_paths {
    //             println!("cargo:rustc-link-search=native={}", path.display());
    //         }
    //         println!("cargo:rustc-link-lib=static=x264");
    //     } else {
    //         println!("cargo:warning=x264未找到，请先运行程序安装vcpkg和x264");
    //     }
    // } else {
    //     println!("cargo:warning=vcpkg未配置，请先运行程序安装vcpkg");
    // }
}

