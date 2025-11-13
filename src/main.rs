mod vcpkg_manager;
mod addon_preparer;

use vcpkg_manager::VcpkgManager;
use addon_preparer::AddonPreparer;

fn main() {
    println!("=== vcpkg FFmpeg/x264 安装器 ===\n");
    
    let manager = VcpkgManager::new();
    
    // 步骤1: 安装vcpkg
    match manager.install_vcpkg() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ vcpkg安装失败: {}", e);
            std::process::exit(1);
        }
    }
    
    // 步骤2: 安装ffmpeg和x264
    match manager.install_packages() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ 包安装失败: {}", e);
            std::process::exit(1);
        }
    }
    
    // 步骤3: 解压ffmpeg包
    match manager.extract_ffmpeg() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ ffmpeg解压失败: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n=== 安装完成 ===");
    println!("vcpkg根目录: {}", manager.get_vcpkg_root().display());
    println!("vcpkg可执行文件: {}", manager.get_vcpkg_exe().display());
    
    // 显示ffmpeg解压目录
    if let Some(ffmpeg_dir) = manager.is_ffmpeg_extracted() {
        println!("ffmpeg项目目录: {}", ffmpeg_dir.display());
    }
    
    // 步骤4: 准备 Node.js addon 源码
    let addon_preparer = AddonPreparer::new();
    match addon_preparer.prepare_addon_source() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ addon准备失败: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n=== 所有步骤完成 ===");
    println!("addon源码目录: {}", addon_preparer.get_addon_src_dir().display());
    println!("\n下一步：");
    println!("1. 进入 addon_src 目录");
    println!("2. 运行: npm install");
    println!("3. 运行: npm run build");
}
