use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
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
    
    /// Prepare addon source code
    pub fn prepare_addon_source(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Preparing Node.js addon source code...");
        
        if !self.addon_src_dir.exists() {
            fs::create_dir_all(&self.addon_src_dir)?;
            println!("✓ Created addon_src directory");
        }
        
        self.create_config_h()?;
        self.modify_opt_common_c()?;
        self.copy_and_modify_ffmpeg_c()?;
        self.create_binding_c()?;
        
        println!("✓ Node.js addon source code preparation completed");
        Ok(())
    }
    
    /// Create config.h file (required for ffmpeg compilation)
    fn create_config_h(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_h_path = self.ffmpeg_source_dir.join("config.h");
        
        if config_h_path.exists() {
            println!("✓ config.h already exists, skipping creation");
            return Ok(());
        }
        
        let config_h_content = r#"/* config.h - Generated for Windows build */
#ifndef CONFIG_H
#define CONFIG_H

/* Windows specific defines */
#define HAVE_IO_H 1
#define HAVE_UNISTD_H 0
#define HAVE_SYS_RESOURCE_H 0
#define HAVE_GETPROCESSTIMES 1
#define HAVE_GETPROCESSMEMORYINFO 1
#define HAVE_SETCONSOLECTRLHANDLER 1
#define HAVE_SYS_SELECT_H 0
#define HAVE_TERMIOS_H 0
#define HAVE_KBHIT 1
#define HAVE_PEEKNAMEDPIPE 1
#define HAVE_GETSTDHANDLE 1
#define HAVE_GETRUSAGE 0

/* FFmpeg components */
#define CONFIG_AVUTIL 1
#define CONFIG_AVCODEC 1
#define CONFIG_AVFORMAT 1
#define CONFIG_AVDEVICE 1
#define CONFIG_AVFILTER 1
#define CONFIG_SWSCALE 1
#define CONFIG_SWRESAMPLE 1
#define CONFIG_POSTPROC 0

/* Architecture */
#define ARCH_X86_32 0
#define ARCH_X86_64 1

/* Threading */
#define HAVE_PTHREADS 0
#define HAVE_W32THREADS 1

/* Endianness */
#define HAVE_BIGENDIAN 0

/* Math functions - MSVC provides these as intrinsics */
#define HAVE_LRINT 1
#define HAVE_LRINTF 1

/* FFmpeg data directory - empty for Node.js addon */
#define FFMPEG_DATADIR ""
#define AVCONV_DATADIR ""

/* Build configuration */
#define CONFIG_THIS_YEAR 2025
#define FFMPEG_CONFIGURATION "Windows build for Node.js addon"
#define CC_IDENT "MSVC"
#define FFMPEG_VERSION "N/A"

#endif /* CONFIG_H */
"#;
        
        fs::write(&config_h_path, config_h_content)?;
        println!("✓ config.h created: {}", config_h_path.display());
        Ok(())
    }
    
    /// Copy and modify ffmpeg.c
    fn copy_and_modify_ffmpeg_c(&self) -> Result<(), Box<dyn std::error::Error>> {
        let source_file = self.ffmpeg_source_dir.join("fftools").join("ffmpeg.c");
        let target_file = self.addon_src_dir.join("ffmpeg.c");
        
        if !source_file.exists() {
            return Err(format!("Source file does not exist: {}", source_file.display()).into());
        }
        
        println!("Copying and modifying ffmpeg.c...");
        
        let content = fs::read_to_string(&source_file)?;
        let modified_content = self.modify_ffmpeg_c_content(&content)?;
        fs::write(&target_file, modified_content)?;
        
        println!("✓ ffmpeg.c copied and modified to: {}", target_file.display());
        Ok(())
    }
    
    /// Modify ffmpeg.c content
    fn modify_ffmpeg_c_content(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut modified = content.to_string();
        
        modified = modified.replace("static int transcode(Scheduler *sch)", "int transcode(Scheduler *sch)");
        modified = modified.replace("static void ffmpeg_cleanup(int ret)", "void ffmpeg_cleanup(int ret)");
        modified = self.remove_main_function(&modified)?;
        modified = self.add_napi_include(&modified)?;
        modified = self.add_ffmpeg_run_function(&modified)?;
        
        Ok(modified)
    }
    
    /// Add node_api.h include
    fn add_napi_include(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let include_marker = "#include \"ffmpeg_utils.h\"";
        if content.contains("#include <node_api.h>") {
            return Ok(content.to_string());
        }
        
        if let Some(pos) = content.find(include_marker) {
            let insert_pos = pos + include_marker.len();
            let result = format!("{}\n#include <node_api.h>\n{}", 
                &content[..insert_pos],
                &content[insert_pos..]
            );
            return Ok(result);
        }
        
        Ok(content.to_string())
    }
    
    /// Remove main function
    fn remove_main_function(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let main_start = "int main(int argc, char **argv)";
        if let Some(start_pos) = content.find(main_start) {
            let func_start = content[start_pos..].find('{');
            if let Some(func_start_pos) = func_start {
                let brace_start = start_pos + func_start_pos;
                
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
                    let before = &content[..start_pos];
                    let after = &content[end..];
                    
                    let result = format!("{}/*\n * Main function removed for Node.js addon\n * Use ffmpeg_run() instead\n */\n{}", 
                        before.trim_end(), 
                        after.trim_start()
                    );
                    
                    return Ok(result);
                }
            }
        }
        
        Ok(content.to_string())
    }
    
    /// Add ffmpeg_run function (N-API implementation for Node.js addon)
    fn add_ffmpeg_run_function(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 检查是否已经存在 ffmpeg_run 函数
        if content.contains("napi_value ffmpeg_run") {
            return Ok(content.to_string());
        }
        
        let run_function = r#"

/**
 * Run ffmpeg with arguments (N-API function for Node.js addon)
 * This function replaces the main() function for use in Node.js addon
 */
napi_value ffmpeg_run(napi_env env, napi_callback_info info)
{
    napi_status status;
    size_t argc = 1;
    napi_value argv[1];
    napi_value result;
    Scheduler *sch = NULL;
    int ret;
    BenchmarkTimeStamps ti;
    
    // 获取参数
    status = napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to get callback info");
        return NULL;
    }
    
    if (argc < 1) {
        napi_throw_type_error(env, NULL, "Expected an array of arguments");
        return NULL;
    }
    
    // 检查第一个参数是否为数组
    napi_valuetype valuetype;
    status = napi_typeof(env, argv[0], &valuetype);
    if (status != napi_ok || valuetype != napi_object) {
        napi_throw_type_error(env, NULL, "Expected an array of arguments");
        return NULL;
    }
    
    // 检查是否为数组
    bool is_array;
    status = napi_is_array(env, argv[0], &is_array);
    if (status != napi_ok || !is_array) {
        napi_throw_type_error(env, NULL, "Expected an array of arguments");
        return NULL;
    }
    
    // 获取数组长度
    uint32_t array_length;
    status = napi_get_array_length(env, argv[0], &array_length);
    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to get array length");
        return NULL;
    }
    
    // 分配内存存储字符串参数
    // 需要额外一个位置给"ffmpeg"程序名
    int total_args = (int)array_length + 1;
    char **argv_ptr = (char **)av_mallocz(sizeof(char *) * total_args);
    if (!argv_ptr) {
        napi_throw_error(env, NULL, "Failed to allocate memory");
        return NULL;
    }
    
    // 存储字符串内容的内存（需要持久化）
    char **str_storage = (char **)av_mallocz(sizeof(char *) * total_args);
    if (!str_storage) {
        av_free(argv_ptr);
        napi_throw_error(env, NULL, "Failed to allocate memory");
        return NULL;
    }
    
    // 第一个参数是程序名
    argv_ptr[0] = "ffmpeg";
    
    // 从JavaScript数组提取字符串参数
    for (uint32_t i = 0; i < array_length; i++) {
        napi_value element;
        status = napi_get_element(env, argv[0], i, &element);
        if (status != napi_ok) {
            // 清理内存
            for (int j = 0; j < i + 1; j++) {
                if (str_storage[j]) av_free(str_storage[j]);
            }
            av_free(str_storage);
            av_free(argv_ptr);
            napi_throw_error(env, NULL, "Failed to get array element");
            return NULL;
        }
        
        // 获取字符串值
        size_t str_len;
        status = napi_get_value_string_utf8(env, element, NULL, 0, &str_len);
        if (status != napi_ok) {
            // 清理内存
            for (int j = 0; j < i + 1; j++) {
                if (str_storage[j]) av_free(str_storage[j]);
            }
            av_free(str_storage);
            av_free(argv_ptr);
            napi_throw_type_error(env, NULL, "Array element must be a string");
            return NULL;
        }
        
        // 分配内存并复制字符串
        str_storage[i + 1] = (char *)av_mallocz(str_len + 1);
        if (!str_storage[i + 1]) {
            // 清理内存
            for (int j = 0; j < i + 1; j++) {
                if (str_storage[j]) av_free(str_storage[j]);
            }
            av_free(str_storage);
            av_free(argv_ptr);
            napi_throw_error(env, NULL, "Failed to allocate memory for string");
            return NULL;
        }
        
        size_t copied;
        status = napi_get_value_string_utf8(env, element, str_storage[i + 1], str_len + 1, &copied);
        if (status != napi_ok) {
            // 清理内存
            for (int j = 0; j < i + 2; j++) {
                if (str_storage[j]) av_free(str_storage[j]);
            }
            av_free(str_storage);
            av_free(argv_ptr);
            napi_throw_error(env, NULL, "Failed to get string value");
            return NULL;
        }
        
        argv_ptr[i + 1] = str_storage[i + 1];
    }
    
    // 调用ffmpeg核心逻辑
    init_dynload();
    
    setvbuf(stderr, NULL, _IONBF, 0);
    
    av_log_set_flags(AV_LOG_SKIP_REPEATED);
    parse_loglevel(total_args, argv_ptr, options);
    
#if CONFIG_AVDEVICE
    avdevice_register_all();
#endif
    avformat_network_init();
    
    sch = sch_alloc();
    if (!sch) {
        ret = AVERROR(ENOMEM);
        goto finish;
    }
    
    ret = ffmpeg_parse_options(total_args, argv_ptr, sch);
    if (ret < 0)
        goto finish;
    
    if (nb_output_files <= 0 && nb_input_files == 0) {
        av_log(NULL, AV_LOG_WARNING, "No input or output files specified\n");
        ret = 1;
        goto finish;
    }
    
    if (nb_output_files <= 0) {
        av_log(NULL, AV_LOG_FATAL, "At least one output file must be specified\n");
        ret = 1;
        goto finish;
    }
    
    current_time = ti = get_benchmark_time_stamps();
    ret = transcode(sch);
    if (ret >= 0 && do_benchmark) {
        int64_t utime, stime, rtime;
        current_time = get_benchmark_time_stamps();
        utime = current_time.user_usec - ti.user_usec;
        stime = current_time.sys_usec  - ti.sys_usec;
        rtime = current_time.real_usec - ti.real_usec;
        av_log(NULL, AV_LOG_INFO,
               "bench: utime=%0.3fs stime=%0.3fs rtime=%0.3fs\n",
               utime / 1000000.0, stime / 1000000.0, rtime / 1000000.0);
    }
    
    ret = received_nb_signals                 ? 255 :
          (ret == FFMPEG_ERROR_RATE_EXCEEDED) ?  69 : ret;
    
finish:
    if (ret == AVERROR_EXIT)
        ret = 0;
    
    ffmpeg_cleanup(ret);
    
    sch_free(&sch);
    
    // 清理字符串内存
    for (int i = 1; i < total_args; i++) {
        if (str_storage[i]) av_free(str_storage[i]);
    }
    av_free(str_storage);
    av_free(argv_ptr);
    
    // 返回结果
    status = napi_create_int32(env, ret, &result);
    if (status != napi_ok) {
        return NULL;
    }
    
    return result;
}
"#;
        
        Ok(format!("{}{}", content, run_function))
    }
    
    /// Modify opt_common.c to add conditional compilation for postproc
    fn modify_opt_common_c(&self) -> Result<(), Box<dyn std::error::Error>> {
        let opt_common_c_path = self.ffmpeg_source_dir.join("fftools").join("opt_common.c");
        
        if !opt_common_c_path.exists() {
            println!("⚠ opt_common.c not found, skipping modification");
            return Ok(());
        }
        
        let content = fs::read_to_string(&opt_common_c_path)?;
        
        // 检查是否已经修改过
        if content.contains("#if CONFIG_POSTPROC") && content.contains("PRINT_LIB_INFO(postproc") {
            println!("✓ opt_common.c already modified, skipping");
            return Ok(());
        }
        
        // 查找 print_all_libs_info 函数中的 postproc 行
        let pattern = "    PRINT_LIB_INFO(postproc,   POSTPROC,   flags, level);";
        if let Some(pos) = content.find(pattern) {
            let before = &content[..pos];
            let after = &content[pos + pattern.len()..];
            
            let modified = format!("{}#if CONFIG_POSTPROC\n    PRINT_LIB_INFO(postproc,   POSTPROC,   flags, level);\n#endif{}", 
                before, after);
            
            fs::write(&opt_common_c_path, modified)?;
            println!("✓ opt_common.c modified: added CONFIG_POSTPROC conditional compilation");
        } else {
            println!("⚠ Could not find postproc line in opt_common.c, skipping modification");
        }
        
        Ok(())
    }
    
    /// Create binding.c
    fn create_binding_c(&self) -> Result<(), Box<dyn std::error::Error>> {
        let binding_c_path = self.addon_src_dir.join("binding.c");
        
        let binding_c_content = r#"#include <node_api.h>

// 声明ffmpeg.c中的napi函数
extern napi_value ffmpeg_run(napi_env env, napi_callback_info info);

napi_value Init(napi_env env, napi_value exports)
{
    napi_status status;
    napi_value fn;
    
    // 创建run函数
    status = napi_create_function(env, NULL, 0, ffmpeg_run, NULL, &fn);
    if (status != napi_ok) {
        return NULL;
    }
    
    // 将run函数添加到exports对象
    status = napi_set_named_property(env, exports, "run", fn);
    if (status != napi_ok) {
        return NULL;
    }
    
    return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
"#;
        
        fs::write(&binding_c_path, binding_c_content)?;
        println!("✓ binding.c created: {}", binding_c_path.display());
        Ok(())
    }
    
    /// Get addon_src directory
    pub fn get_addon_src_dir(&self) -> &Path {
        &self.addon_src_dir
    }
}

