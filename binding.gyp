{
  "targets": [
    {
      "target_name": "ffmpeg_node",
      "sources": [
        "./addon_src/binding.c",
        "./addon_src/ffmpeg.c",
        "./ffmpeg/fftools/cmdutils.c",
        "./ffmpeg/fftools/ffmpeg_dec.c",
        "./ffmpeg/fftools/ffmpeg_demux.c",
        "./ffmpeg/fftools/ffmpeg_enc.c",
        "./ffmpeg/fftools/ffmpeg_filter.c",
        "./ffmpeg/fftools/ffmpeg_hw.c",
        "./ffmpeg/fftools/ffmpeg_mux_init.c",
        "./ffmpeg/fftools/ffmpeg_mux.c",
        "./ffmpeg/fftools/ffmpeg_opt.c",
        "./ffmpeg/fftools/ffmpeg_sched.c",
        "./ffmpeg/fftools/opt_common.c",
        "./ffmpeg/fftools/sync_queue.c",
        "./ffmpeg/fftools/thread_queue.c",
        "./ffmpeg/fftools/objpool.c"
      ],
      "include_dirs": [
        "<!@(node -p \"require('node-addon-api').include\")",
        "<!@(node -p \"require('path').dirname(process.execPath) + '/include/node'\")",
        "./ffmpeg",
        "./ffmpeg/fftools",
        "./ffmpeg/compat/atomics/win32",
        "./vcpkg/installed/x64-windows-static/include"
      ],
      "libraries": [
        "-L./vcpkg/installed/x64-windows-static/lib",
        "-lavcodec",
        "-lavformat",
        "-lavutil",
        "-lavfilter",
        "-lswscale",
        "-lswresample",
        "-lavdevice",
        "-lx264"
      ],
      "msvs_settings": {
        "VCCLCompilerTool": {
          "ExceptionHandling": 0,
          "CompileAs": "1",
          "AdditionalOptions": [ "/std:c++17", "/TC" ]
        }
      },
      "conditions": [
        ["OS=='win'", {
          "libraries": [
            "-L./vcpkg/installed/x64-windows-static/lib",
            "avcodec.lib",
            "avformat.lib",
            "avutil.lib",
            "avfilter.lib",
            "swscale.lib",
            "swresample.lib",
            "avdevice.lib",
            "x264.lib"
          ]
        }]
      ]
    }
  ]
}
