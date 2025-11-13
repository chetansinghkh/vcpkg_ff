mod vcpkg_manager;
mod addon_preparer;

use vcpkg_manager::VcpkgManager;
use addon_preparer::AddonPreparer;

fn main() {
    println!("=== vcpkg FFmpeg/x264 Installer ===\n");
    
    let manager = VcpkgManager::new();
    
    match manager.install_vcpkg() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ vcpkg installation failed: {}", e);
            std::process::exit(1);
        }
    }
    
    match manager.install_packages() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ Package installation failed: {}", e);
            std::process::exit(1);
        }
    }
    
    match manager.extract_ffmpeg() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ ffmpeg extraction failed: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n=== Installation Complete ===");
    println!("vcpkg root: {}", manager.get_vcpkg_root().display());
    println!("vcpkg executable: {}", manager.get_vcpkg_exe().display());
    
    if let Some(ffmpeg_dir) = manager.is_ffmpeg_extracted() {
        println!("ffmpeg project directory: {}", ffmpeg_dir.display());
    }
    
    let addon_preparer = AddonPreparer::new();
    match addon_preparer.prepare_addon_source() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("✗ Addon preparation failed: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n=== All Steps Completed ===");
    println!("addon source directory: {}", addon_preparer.get_addon_src_dir().display());
}
