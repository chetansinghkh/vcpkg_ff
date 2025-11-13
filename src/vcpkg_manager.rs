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
        // 在运行时，CARGO_MANIFEST_DIR可能不可用，使用当前工作目录
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
    
    /// 检查vcpkg是否已安装
    pub fn is_installed(&self) -> bool {
        self.vcpkg_exe.exists()
    }
    
    /// 检查git是否可用
    fn check_git(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .arg("--version")
            .output()?;
        
        if !output.status.success() {
            return Err("git未安装或不在PATH中，请先安装git".into());
        }
        
        Ok(())
    }
    
    /// 安装vcpkg
    pub fn install_vcpkg(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_installed() {
            println!("✓ vcpkg已安装，跳过安装步骤");
            return Ok(());
        }
        
        println!("检查git...");
        self.check_git()?;
        
        println!("开始安装vcpkg到: {}", self.vcpkg_root.display());
        
        // 如果目录已存在但不是vcpkg，先删除
        if self.vcpkg_root.exists() {
            println!("清理现有目录...");
            fs::remove_dir_all(&self.vcpkg_root)?;
        }
        
        // 创建父目录
        if let Some(parent) = self.vcpkg_root.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 克隆vcpkg仓库
        println!("正在克隆vcpkg仓库（这可能需要几分钟）...");
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
            return Err("git clone vcpkg失败".into());
        }
        
        // 运行bootstrap脚本
        println!("正在运行bootstrap脚本...");
        let bootstrap_script = self.vcpkg_root.join("bootstrap-vcpkg.bat");
        let status = Command::new(&bootstrap_script)
            .current_dir(&self.vcpkg_root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        
        if !status.success() {
            return Err("vcpkg bootstrap失败".into());
        }
        
        if !self.vcpkg_exe.exists() {
            return Err("vcpkg.exe未生成，bootstrap可能失败".into());
        }
        
        println!("vcpkg安装完成！");
        Ok(())
    }
    
    /// 检查包是否已安装
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
    
    /// 安装ffmpeg和x264（静态库版本）
    pub fn install_packages(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_installed() {
            return Err("vcpkg未安装，请先调用install_vcpkg()".into());
        }
        
        let x264_installed = self.is_package_installed("x264");
        let ffmpeg_installed = self.is_package_installed("ffmpeg");
        
        // 如果所有包都已安装，直接返回
        if x264_installed && ffmpeg_installed {
            println!("✓ x264静态库已安装");
            println!("✓ ffmpeg静态库已安装");
            return Ok(());
        }
        
        println!("开始安装ffmpeg和x264静态库...");
        println!("注意：这可能需要较长时间（10-30分钟），请耐心等待...");
        
        // 安装x264（静态库）
        if x264_installed {
            println!("✓ x264静态库已安装，跳过安装步骤");
        } else {
            println!("正在安装x264:x64-windows-static（这可能需要几分钟）...");
            let status = Command::new(&self.vcpkg_exe)
                .args(&[
                    "install",
                    "x264:x64-windows-static",
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;
            
            if !status.success() {
                return Err("安装x264失败".into());
            }
            println!("✓ x264安装完成！");
        }
        
        // 安装ffmpeg（静态库，依赖x264）
        if ffmpeg_installed {
            println!("✓ ffmpeg静态库已安装，跳过安装步骤");
        } else {
            println!("正在安装ffmpeg:x64-windows-static（这可能需要10-20分钟）...");
            let status = Command::new(&self.vcpkg_exe)
                .args(&[
                    "install",
                    "ffmpeg:x64-windows-static",
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;
            
            if !status.success() {
                return Err("安装ffmpeg失败".into());
            }
            println!("✓ ffmpeg安装完成！");
        }
        
        println!("✓ 所有包安装完成！");
        Ok(())
    }
    
    /// 获取vcpkg根目录
    pub fn get_vcpkg_root(&self) -> &Path {
        &self.vcpkg_root
    }
    
    /// 获取vcpkg可执行文件路径
    pub fn get_vcpkg_exe(&self) -> &Path {
        &self.vcpkg_exe
    }
    
    /// 检查ffmpeg包是否已解压，返回解压后的文件夹路径
    pub fn is_ffmpeg_extracted(&self) -> Option<PathBuf> {
        let output_dir = self.get_output_dir();
        let ffmpeg_dir = output_dir.join("ffmpeg");
        
        if ffmpeg_dir.exists() && ffmpeg_dir.is_dir() {
            return Some(ffmpeg_dir);
        }
        
        None
    }
    
    /// 获取输出目录（运行目录）
    fn get_output_dir(&self) -> PathBuf {
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(&manifest_dir)
        } else {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
        }
    }
    
    /// 查找ffmpeg tar.gz文件
    fn find_ffmpeg_archive(&self) -> Option<PathBuf> {
        let downloads_dir = self.vcpkg_root.join("downloads");
        if !downloads_dir.exists() {
            return None;
        }
        
        // 查找ffmpeg相关的tar.gz文件
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
    
    /// 解压ffmpeg包到运行目录
    pub fn extract_ffmpeg(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(extracted_dir) = self.is_ffmpeg_extracted() {
            println!("✓ ffmpeg项目已导出，跳过解压步骤");
            println!("  导出目录: {}", extracted_dir.display());
            return Ok(());
        }
        
        let archive_path = match self.find_ffmpeg_archive() {
            Some(path) => path,
            None => {
                return Err("未找到ffmpeg tar.gz文件，请先安装ffmpeg包".into());
            }
        };
        
        println!("正在解压ffmpeg包: {}", archive_path.display());
        
        let output_dir = self.get_output_dir();
        
        // 创建临时目录用于解压
        let temp_dir = output_dir.join(".ffmpeg_temp");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;
        
        // 打开tar.gz文件
        let file = File::open(&archive_path)?;
        let gz_decoder = GzDecoder::new(BufReader::new(file));
        let mut archive = Archive::new(gz_decoder);
        
        // 解压到临时目录
        archive.unpack(&temp_dir)?;
        
        // 查找解压后的顶层目录
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
                return Err("解压后未找到顶层目录".into());
            }
        };
        
        // 目标目录路径
        let target_dir = output_dir.join("ffmpeg");
        
        // 如果目标目录已存在，先删除
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
        }
        
        // 重命名/移动目录为ffmpeg
        fs::rename(&extracted_top_dir, &target_dir)?;
        
        // 清理临时目录
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        
        println!("✓ ffmpeg项目已成功导出到: {}", target_dir.display());
        Ok(())
    }
}

