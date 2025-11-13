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
        "<!@(node -p \"process.platform === 'win32' ? './ffmpeg/compat/atomics/win32' : ''\")",
        "<!@(node -p \"process.platform === 'win32' ? './vcpkg/installed/x64-windows-static/include' : (process.platform === 'darwin' ? './vcpkg/installed/x64-osx/include' : './vcpkg/installed/x64-linux/include')\")"
      ],
      "msvs_settings": {
        "VCCLCompilerTool": {
          "ExceptionHandling": 0
        },
        "VCLinkerTool": {
          "AdditionalLibraryDirectories": [
            "<(module_root_dir)/vcpkg/installed/x64-windows-static/lib"
          ],
          "AdditionalDependencies": [
            "avcodec.lib",
            "avformat.lib",
            "avutil.lib",
            "avfilter.lib",
            "swscale.lib",
            "swresample.lib",
            "avdevice.lib",
            "libx264.lib",
            "x265-static.lib",
            "vpx.lib",
            "ws2_32.lib",
            "secur32.lib",
            "bcrypt.lib",
            "strmiids.lib",
            "ole32.lib",
            "oleaut32.lib",
            "vfw32.lib",
            "mfplat.lib",
            "mfuuid.lib",
            "shlwapi.lib",
            "user32.lib",
            "gdi32.lib",
            "winmm.lib",
            "psapi.lib"
          ]
        }
      },
      "msvs_configurations": {
        "Release": {
          "msvs_settings": {
            "VCCLCompilerTool": {
              "CompileAs": "1"
            }
          }
        }
      },
      "conditions": [
        ["OS=='mac'", {
          "libraries": [
            "<!@(node -p \"'-L' + require('path').join(process.cwd(), 'vcpkg/installed/x64-osx/lib')\")",
            "-lavcodec",
            "-lavformat",
            "-lavutil",
            "-lavfilter",
            "-lswscale",
            "-lswresample",
            "-lavdevice",
            "-lx264",
            "-lx265",
            "-lvpx"
          ],
          "xcode_settings": {
            "OTHER_CPLUSPLUSFLAGS": ["-std=c++17"],
            "OTHER_LDFLAGS": [
              "<!@(node -p \"'-L' + require('path').join(process.cwd(), 'vcpkg/installed/x64-osx/lib')\")"
            ]
          }
        }],
        ["OS=='linux'", {
          "libraries": [
            "<!@(node -p \"'-L' + require('path').join(process.cwd(), 'vcpkg/installed/x64-linux/lib')\")",
            "-lavcodec",
            "-lavformat",
            "-lavutil",
            "-lavfilter",
            "-lswscale",
            "-lswresample",
            "-lavdevice",
            "-lx264",
            "-lx265",
            "-lvpx"
          ]
        }]
      ]
    }
  ]
}
