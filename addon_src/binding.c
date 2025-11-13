#include <node_api.h>

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
