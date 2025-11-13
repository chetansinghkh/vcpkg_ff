use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use flate2::read::GzDecoder;
use tar::Archive;

pub struct VcpkgManager {
    vcpkg_root: PathBuf,
    vcpkg_exe: PathBuf,
}

impl VcpkgManager {
    pub fn new() -> Self {
        let vcpkg_root = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(&manifest_dir).join("vcpkg")
        } else {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("vcpkg")
        };
        
        let vcpkg_exe = vcpkg_root.join("vcpkg.exe");
        
        Self {
            vcpkg_root,
            vcpkg_exe,
        }
    }
    
    /// Check if vcpkg is installed
    pub fn is_installed(&self) -> bool {
        self.vcpkg_exe.exists()
    }
    
    /// Check if git is available
    fn check_git(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .arg("--version")
            .output()?;
        
        if !output.status.success() {
            return Err("git is not installed or not in PATH, please install git first".into());
        }
        
        Ok(())
    }
    
    /// Install vcpkg
    pub fn install_vcpkg(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_installed() {
            println!("✓ vcpkg already installed, skipping installation");
            return Ok(());
        }
        
        println!("Checking git...");
        self.check_git()?;
        
        println!("Starting vcpkg installation to: {}", self.vcpkg_root.display());
        
        if self.vcpkg_root.exists() {
            println!("Cleaning existing directory...");
            fs::remove_dir_all(&self.vcpkg_root)?;
        }
        
        if let Some(parent) = self.vcpkg_root.parent() {
            fs::create_dir_all(parent)?;
        }
        
        println!("Cloning vcpkg repository (this may take a few minutes)...");
        let status = Command::new("git")
            .args(&[
                "clone",
                "https://github.com/Microsoft/vcpkg.git",
                self.vcpkg_root.to_str().unwrap(),
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        
        if !status.success() {
            return Err("git clone vcpkg failed".into());
        }
        
        println!("Running bootstrap script...");
        let bootstrap_script = self.vcpkg_root.join("bootstrap-vcpkg.bat");
        let status = Command::new(&bootstrap_script)
            .current_dir(&self.vcpkg_root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        
        if !status.success() {
            return Err("vcpkg bootstrap failed".into());
        }
        
        if !self.vcpkg_exe.exists() {
            return Err("vcpkg.exe was not generated, bootstrap may have failed".into());
        }
        
        println!("vcpkg installation completed!");
        Ok(())
    }
    
    /// Check if package is installed
    pub fn is_package_installed(&self, package: &str) -> bool {
        let output = Command::new(&self.vcpkg_exe)
            .args(&["list", package])
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout.contains(package) && stdout.contains("x64-windows-static");
            }
        }
        
        false
    }
    
    /// Install ffmpeg and x264 (static library version)
    pub fn install_packages(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_installed() {
            return Err("vcpkg is not installed, please call install_vcpkg() first".into());
        }
        
        let x264_installed = self.is_package_installed("x264");
        let ffmpeg_installed = self.is_package_installed("ffmpeg");
        
        if x264_installed && ffmpeg_installed {
            println!("✓ x264 static library already installed");
            println!("✓ ffmpeg static library already installed");
            return Ok(());
        }
        
        println!("Starting installation of ffmpeg and x264 static libraries...");
        println!("Note: This may take a long time (10-30 minutes), please wait patiently...");
        
        if x264_installed {
            println!("✓ x264 static library already installed, skipping installation");
        } else {
            println!("Installing x264:x64-windows-static (this may take a few minutes)...");
            let status = Command::new(&self.vcpkg_exe)
                .args(&[
                    "install",
                    "x264:x64-windows-static",
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;
            
            if !status.success() {
                return Err("x264 installation failed".into());
            }
            println!("✓ x264 installation completed!");
        }
        
        if ffmpeg_installed {
            println!("✓ ffmpeg static library already installed, skipping installation");
        } else {
            println!("Installing ffmpeg:x64-windows-static (this may take 10-20 minutes)...");
            let status = Command::new(&self.vcpkg_exe)
                .args(&[
                    "install",
                    "ffmpeg:x64-windows-static",
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;
            
            if !status.success() {
                return Err("ffmpeg installation failed".into());
            }
            println!("✓ ffmpeg installation completed!");
        }
        
        println!("✓ All packages installation completed!");
        Ok(())
    }
    
    /// Get vcpkg root directory
    pub fn get_vcpkg_root(&self) -> &Path {
        &self.vcpkg_root
    }
    
    /// Get vcpkg executable path
    pub fn get_vcpkg_exe(&self) -> &Path {
        &self.vcpkg_exe
    }
    
    /// Check if ffmpeg package is extracted, returns extracted folder path
    pub fn is_ffmpeg_extracted(&self) -> Option<PathBuf> {
        let output_dir = self.get_output_dir();
        let ffmpeg_dir = output_dir.join("ffmpeg");
        
        if ffmpeg_dir.exists() && ffmpeg_dir.is_dir() {
            return Some(ffmpeg_dir);
        }
        
        None
    }
    
    /// Get output directory (runtime directory)
    fn get_output_dir(&self) -> PathBuf {
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(&manifest_dir)
        } else {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
        }
    }
    
    /// Find ffmpeg tar.gz file
    fn find_ffmpeg_archive(&self) -> Option<PathBuf> {
        let downloads_dir = self.vcpkg_root.join("downloads");
        if !downloads_dir.exists() {
            return None;
        }
        
        if let Ok(entries) = fs::read_dir(&downloads_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("ffmpeg") && name.ends_with(".tar.gz") {
                        return Some(path);
                    }
                }
            }
        }
        
        None
    }
    
    /// Extract ffmpeg package to runtime directory
    pub fn extract_ffmpeg(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(extracted_dir) = self.is_ffmpeg_extracted() {
            println!("✓ ffmpeg project already exported, skipping extraction");
            println!("  Export directory: {}", extracted_dir.display());
            return Ok(());
        }
        
        let archive_path = match self.find_ffmpeg_archive() {
            Some(path) => path,
            None => {
                return Err("ffmpeg tar.gz file not found, please install ffmpeg package first".into());
            }
        };
        
        println!("Extracting ffmpeg package: {}", archive_path.display());
        
        let output_dir = self.get_output_dir();
        
        let temp_dir = output_dir.join(".ffmpeg_temp");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;
        
        let file = File::open(&archive_path)?;
        let gz_decoder = GzDecoder::new(BufReader::new(file));
        let mut archive = Archive::new(gz_decoder);
        
        archive.unpack(&temp_dir)?;
        
        let mut extracted_top_dir = None;
        if let Ok(entries) = temp_dir.read_dir() {
            for entry in entries.flatten() {
                if let Ok(entry_type) = entry.file_type() {
                    if entry_type.is_dir() {
                        extracted_top_dir = Some(entry.path());
                        break;
                    }
                }
            }
        }
        
        let extracted_top_dir = match extracted_top_dir {
            Some(dir) => dir,
            None => {
                fs::remove_dir_all(&temp_dir)?;
                return Err("Top-level directory not found after extraction".into());
            }
        };
        
        let target_dir = output_dir.join("ffmpeg");
        
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
        }
        
        fs::rename(&extracted_top_dir, &target_dir)?;
        
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        
        println!("✓ ffmpeg project successfully exported to: {}", target_dir.display());
        Ok(())
    }
}

