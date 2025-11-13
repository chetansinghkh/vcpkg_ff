use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct AddonPreparer {
    ffmpeg_source_dir: PathBuf,
    addon_src_dir: PathBuf,
    vcpkg_root: PathBuf,
}

impl AddonPreparer {
    pub fn new() -> Self {
        let base_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(&manifest_dir)
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };
        
        let ffmpeg_source_dir = base_dir.join("ffmpeg");
        let addon_src_dir = base_dir.join("addon_src");
        let vcpkg_root = base_dir.join("vcpkg");
        
        Self {
            ffmpeg_source_dir,
            addon_src_dir,
            vcpkg_root,
        }
    }
    
    /// 准备 addon 源码
    pub fn prepare_addon_source(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("开始准备 Node.js addon 源码...");
        
        // 确保 addon_src 目录存在
        if !self.addon_src_dir.exists() {
            fs::create_dir_all(&self.addon_src_dir)?;
            println!("✓ 创建 addon_src 目录");
        }
        
        // 步骤1: 复制并修改 ffmpeg.c
        self.copy_and_modify_ffmpeg_c()?;
        
        // 步骤2: 创建 binding.gyp
        self.create_binding_gyp()?;
        
        // 步骤3: 创建 binding.cpp
        self.create_binding_cpp()?;
        
        // 步骤4: 创建 package.json（如果需要）
        self.create_package_json()?;
        
        println!("✓ Node.js addon 源码准备完成");
        Ok(())
    }
    
    /// 复制并修改 ffmpeg.c
    fn copy_and_modify_ffmpeg_c(&self) -> Result<(), Box<dyn std::error::Error>> {
        let source_file = self.ffmpeg_source_dir.join("fftools").join("ffmpeg.c");
        let target_file = self.addon_src_dir.join("ffmpeg.c");
        
        if !source_file.exists() {
            return Err(format!("源文件不存在: {}", source_file.display()).into());
        }
        
        println!("正在复制并修改 ffmpeg.c...");
        
        // 读取源文件
        let content = fs::read_to_string(&source_file)?;
        
        // 修改内容
        let modified_content = self.modify_ffmpeg_c_content(&content)?;
        
        // 写入目标文件
        fs::write(&target_file, modified_content)?;
        
        println!("✓ ffmpeg.c 已复制并修改到: {}", target_file.display());
        Ok(())
    }
    
    /// 修改 ffmpeg.c 的内容
    fn modify_ffmpeg_c_content(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut modified = content.to_string();
        
        // 1. 将 transcode 从 static 改为公开函数
        modified = modified.replace("static int transcode(Scheduler *sch)", "int transcode(Scheduler *sch)");
        
        // 2. 将 ffmpeg_cleanup 从 static 改为公开函数
        modified = modified.replace("static void ffmpeg_cleanup(int ret)", "void ffmpeg_cleanup(int ret)");
        
        // 3. 删除 main 函数
        modified = self.remove_main_function(&modified)?;
        
        // 4. 确保必要的函数是公开的（已在步骤1-2完成）
        modified = self.add_ffmpeg_run_function(&modified)?;
        
        Ok(modified)
    }
    
    /// 删除 main 函数
    fn remove_main_function(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 查找 main 函数的开始位置
        let main_start = "int main(int argc, char **argv)";
        if let Some(start_pos) = content.find(main_start) {
            // 找到函数体的开始大括号
            let func_start = content[start_pos..].find('{');
            if let Some(func_start_pos) = func_start {
                let brace_start = start_pos + func_start_pos;
                
                // 使用简单的字符串匹配找到对应的结束大括号
                let mut brace_count = 0;
                let mut in_string = false;
                let mut escape_next = false;
                let mut end_pos = None;
                
                let bytes = content[brace_start..].as_bytes();
                for (i, &byte) in bytes.iter().enumerate() {
                    let ch = byte as char;
                    
                    if escape_next {
                        escape_next = false;
                        continue;
                    }
                    
                    match ch {
                        '\\' if in_string => escape_next = true,
                        '"' => in_string = !in_string,
                        '{' if !in_string => {
                            brace_count += 1;
                        }
                        '}' if !in_string => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                end_pos = Some(brace_start + i + 1);
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                
                if let Some(end) = end_pos {
                    // 删除 main 函数，只保留前后的内容
                    let before = &content[..start_pos];
                    let after = &content[end..];
                    
                    // 添加注释说明 main 函数已被删除
                    let result = format!("{}/*\n * Main function removed for Node.js addon\n * Use ffmpeg_run() instead\n */\n{}", 
                        before.trim_end(), 
                        after.trim_start()
                    );
                    
                    return Ok(result);
                }
            }
        }
        
        // 如果找不到或匹配失败，直接返回原内容
        Ok(content.to_string())
    }
    
    /// 添加 ffmpeg_run 辅助函数（用于 Node.js addon）
    fn add_ffmpeg_run_function(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 不需要添加 C 函数，所有逻辑都在 binding.cpp 中处理
        // 只需要确保 transcode 和 ffmpeg_cleanup 是公开的即可
        Ok(content.to_string())
    }
    
    /// 创建 binding.gyp
    fn create_binding_gyp(&self) -> Result<(), Box<dyn std::error::Error>> {
        let binding_gyp_path = self.addon_src_dir.join("binding.gyp");
        
        // 获取 vcpkg 安装的库路径
        let vcpkg_installed = self.vcpkg_root.join("installed").join("x64-windows-static");
        let lib_dir = vcpkg_installed.join("lib");
        let include_dir = vcpkg_installed.join("include");
        
        // 将 Windows 路径转换为正斜杠格式（node-gyp 支持）
        let lib_dir_str = lib_dir.to_string_lossy().replace("\\", "/");
        let include_dir_str = include_dir.to_string_lossy().replace("\\", "/");
        
        let binding_gyp_content = format!(r#"{{
  "targets": [
    {{
      "target_name": "ffmpeg_node",
      "sources": [
        "binding.cpp",
        "ffmpeg.c",
        "../ffmpeg/fftools/cmdutils.c",
        "../ffmpeg/fftools/ffmpeg_dec.c",
        "../ffmpeg/fftools/ffmpeg_demux.c",
        "../ffmpeg/fftools/ffmpeg_enc.c",
        "../ffmpeg/fftools/ffmpeg_filter.c",
        "../ffmpeg/fftools/ffmpeg_hw.c",
        "../ffmpeg/fftools/ffmpeg_mux_init.c",
        "../ffmpeg/fftools/ffmpeg_mux.c",
        "../ffmpeg/fftools/ffmpeg_opt.c",
        "../ffmpeg/fftools/ffmpeg_sched.c",
        "../ffmpeg/fftools/opt_common.c",
        "../ffmpeg/fftools/sync_queue.c",
        "../ffmpeg/fftools/thread_queue.c",
        "../ffmpeg/fftools/objpool.c"
      ],
      "include_dirs": [
        "<!@(node -p \"require('node-addon-api').include\")",
        "../ffmpeg",
        "../ffmpeg/fftools",
        "{}"
      ],
      "libraries": [
        "-L{}",
        "-lavcodec",
        "-lavformat",
        "-lavutil",
        "-lavfilter",
        "-lswscale",
        "-lswresample",
        "-lavdevice",
        "-lx264"
      ],
      "defines": [
        "NAPI_DISABLE_CPP_EXCEPTIONS"
      ],
      "cflags!": [ "-fno-exceptions" ],
      "cflags_cc!": [ "-fno-exceptions" ],
      "xcode_settings": {{
        "GCC_ENABLE_CPP_EXCEPTIONS": "YES",
        "CLANG_CXX_LIBRARY": "libc++",
        "MACOSX_DEPLOYMENT_TARGET": "10.7"
      }},
      "msvs_settings": {{
        "VCCLCompilerTool": {{
          "ExceptionHandling": 1
        }}
      }},
      "conditions": [
        ["OS=='win'", {{
          "libraries": [
            "-L{}",
            "avcodec.lib",
            "avformat.lib",
            "avutil.lib",
            "avfilter.lib",
            "swscale.lib",
            "swresample.lib",
            "avdevice.lib",
            "x264.lib"
          ]
        }}]
      ]
    }}
  ]
}}
"#, 
            include_dir_str, lib_dir_str, lib_dir_str
        );
        
        fs::write(&binding_gyp_path, binding_gyp_content)?;
        println!("✓ binding.gyp 已创建: {}", binding_gyp_path.display());
        Ok(())
    }
    
    /// 创建 binding.cpp
    fn create_binding_cpp(&self) -> Result<(), Box<dyn std::error::Error>> {
        let binding_cpp_path = self.addon_src_dir.join("binding.cpp");
        
        let binding_cpp_content = r#"#include <napi.h>
#include <vector>
#include <string>
#include <cstring>
#include <cstdio>
#include <cstdint>

// 包含 ffmpeg 头文件
#include "ffmpeg.h"
#include "ffmpeg_sched.h"
#include "cmdutils.h"

// 声明必要的函数和变量
extern "C" {
    // 从 cmdutils.c 中需要的函数
    void init_dynload(void);
    void parse_loglevel(int argc, char **argv, const OptionDef *options);
    void avformat_network_init(void);
    void av_log_set_flags(int flags);
    
    #if CONFIG_AVDEVICE
    void avdevice_register_all(void);
    #endif
    
    // 全局变量
    extern int nb_output_files;
    extern int nb_input_files;
    extern int received_nb_signals;
    extern int do_benchmark;
    extern int64_t current_time;
    extern const OptionDef options[];
    
    // 常量定义
    #ifndef FFMPEG_ERROR_RATE_EXCEEDED
    #define FFMPEG_ERROR_RATE_EXCEEDED 0x45455245  // 'ERE' in ASCII
    #endif
}

// Node.js addon 的 run 函数 - 直接处理所有逻辑
Napi::Value Run(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    // 检查参数
    if (info.Length() < 1 || !info[0].IsArray()) {
        Napi::TypeError::New(env, "Expected an array of arguments")
            .ThrowAsJavaScriptException();
        return env.Null();
    }
    
    // 获取参数数组
    Napi::Array args_array = info[0].As<Napi::Array>();
    
    // 转换为字符串数组
    std::vector<std::string> string_args;
    string_args.push_back("ffmpeg"); // 添加程序名
    
    // 转换 JavaScript 参数
    for (uint32_t i = 0; i < args_array.Length(); i++) {
        Napi::Value val = args_array[i];
        if (val.IsString()) {
            string_args.push_back(val.As<Napi::String>().Utf8Value());
        } else {
            string_args.push_back(val.ToString().Utf8Value());
        }
    }
    
    // 创建 C 风格的参数数组（需要保持字符串的生命周期）
    static thread_local std::vector<std::string> persistent_args;
    persistent_args = string_args;
    
    std::vector<char*> argv;
    for (auto& str : persistent_args) {
        argv.push_back(const_cast<char*>(str.c_str()));
    }
    
    int argc = static_cast<int>(argv.size());
    
    // 初始化 ffmpeg
    init_dynload();
    
    // 设置 stderr 缓冲（Windows 需要）
    #ifdef _WIN32
    setvbuf(stderr, NULL, _IONBF, 0);
    #endif
    
    // 设置日志标志
    av_log_set_flags(AV_LOG_SKIP_REPEATED);
    
    // 解析日志级别
    parse_loglevel(argc, argv.data(), options);
    
    // 注册设备
    #if CONFIG_AVDEVICE
    avdevice_register_all();
    #endif
    
    // 初始化网络
    avformat_network_init();
    
    // 分配调度器
    Scheduler *sch = sch_alloc();
    if (!sch) {
        return Napi::Number::New(env, AVERROR(ENOMEM));
    }
    
    // 解析选项
    int ret = ffmpeg_parse_options(argc, argv.data(), sch);
    if (ret < 0) {
        sch_free(&sch);
        return Napi::Number::New(env, ret);
    }
    
    // 检查输入/输出文件
    if (nb_output_files <= 0 && nb_input_files == 0) {
        sch_free(&sch);
        ffmpeg_cleanup(1);
        return Napi::Number::New(env, 1);
    }
    
    if (nb_output_files <= 0) {
        sch_free(&sch);
        ffmpeg_cleanup(1);
        return Napi::Number::New(env, 1);
    }
    
    // 执行转码
    ret = transcode(sch);
    
    // 处理返回值
    if (ret == AVERROR_EXIT) {
        ret = 0;
    } else if (received_nb_signals) {
        ret = 255;
    } else if (ret == FFMPEG_ERROR_RATE_EXCEEDED) {
        ret = 69;
    }
    
    // 清理
    ffmpeg_cleanup(ret);
    sch_free(&sch);
    
    return Napi::Number::New(env, ret);
}

// 初始化 Node.js addon
Napi::Object Init(Napi::Env env, Napi::Object exports) {
    exports.Set(
        Napi::String::New(env, "run"),
        Napi::Function::New(env, Run)
    );
    return exports;
}

NODE_API_MODULE(ffmpeg_node, Init)
"#;
        
        fs::write(&binding_cpp_path, binding_cpp_content)?;
        println!("✓ binding.cpp 已创建: {}", binding_cpp_path.display());
        Ok(())
    }
    
    /// 创建 package.json
    fn create_package_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        let package_json_path = self.addon_src_dir.join("package.json");
        
        // 如果已存在，检查是否需要更新
        if package_json_path.exists() {
            println!("✓ package.json 已存在，跳过创建");
            return Ok(());
        }
        
        let package_json_content = r#"{
  "name": "ffmpeg-node",
  "version": "1.0.0",
  "description": "FFmpeg Node.js native addon",
  "main": "index.js",
  "gypfile": true,
  "scripts": {
    "build": "node-gyp rebuild",
    "install": "node-gyp rebuild"
  },
  "keywords": [
    "ffmpeg",
    "video",
    "audio",
    "codec"
  ],
  "author": "",
  "license": "LGPL-2.1",
  "dependencies": {
    "node-addon-api": "^7.0.0"
  },
  "devDependencies": {
    "node-gyp": "^10.0.0"
  }
}
"#;
        
        fs::write(&package_json_path, package_json_content)?;
        println!("✓ package.json 已创建: {}", package_json_path.display());
        Ok(())
    }
    
    /// 获取 addon_src 目录
    pub fn get_addon_src_dir(&self) -> &Path {
        &self.addon_src_dir
    }
}

