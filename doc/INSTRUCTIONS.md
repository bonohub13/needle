# How to use needle in OBS Studio
## 0. Preparation (Optional)
- Linux: \
Go to `${HOME}/config/needle` and edit `config.toml` to edit the settings below.
    - Background color (`backgroun_color`)
    - Text format (`format`)
    - Font size (`config.scale`)
    - Font color (`config.color`)
    - Position of text (`config.position`)
    - Framerate visualization (`fps.enable`)
    - Framerate limit (`fps_limit`; default: 30)
- Windows: \
Go to `%APPDATA%\bonohub13\needle\config` and edit `config.toml` to edit the settings below.
    - Background color (`backgroun_color`)
    - Text format (`format`)
    - Font size (`config.scale`)
    - Font color (`config.color`)
    - Position of text (`config.position`)
    - Framerate visualization (`fps.enable`)
    - Framerate limit (`fps_limit`; default: 30)

![Example of Customizing needle](resources/common/edit_needle.png)

## 1. Example for usage in OBS Studio
1. Launch needle and OBS Studio.
2. Select `Window Capture` in `Sources`.
    - ![window capture](en/window_capture.png)
3. Select needle for window source and set the `Capture Method` to `Windows 10 (1903 and up)`
    - ![window capture (settings)](en/window_capture-needle.png)
4. After selecting `needle` in `Sources`, select `filter`.
    - ![filter](en/needle_filter.png)
5. Add `Color Key` to `Effect Filters` and set the background color to `Key Color Type`.
    - ![filter settings](en/needle_filtered.png)
6. DONE!
    - ![end result](en/end_result.png)
