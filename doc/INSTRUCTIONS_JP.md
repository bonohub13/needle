# OBS Studio内でneedleの使用方法
## 0. 準備 (任意)
- Linux: \
`${HOME}/config/needle`ディレクトリ内の`config.toml`で下記の内容が編集可能
    - 背景色 (`backgroun_color`)
    - 時刻のフォーマット (`format`)
    - テキストのサイズ (`config.scale`)
    - テキストの色 (`config.color`)
    - テキストの配置 (`config.position`)
    - フレームレートの表示 (`fps.enable`)
    - フレームレートの上限値設定 (`fps_limit`; デフォルト: 30)
- Linux: \
`%APPDATA%\bonohub13\needle\config`ディレクトリ内の`config.toml`で下記の内容が編集可能
    - 背景色 (`backgroun_color`)
    - 時刻のフォーマット (`format`)
    - テキストのサイズ (`config.scale`)
    - テキストの色 (`config.color`)
    - テキストの配置 (`config.position`)
    - フレームレートの表示 (`fps.enable`)
    - フレームレートの上限値設定 (`fps_limit`; デフォルト: 30)

![needleの編集例](resources/common/edit_config.png)

## 1. OBS Studio内の使用例
1. OBS Studioとneedleを起動.
2. Select `Window Capture` in `Sources`.
    - ![ウィンドウキャプチャ](jp/window_capture_jp.png)
3. Select needle for window source and set the `Capture Method` to `Windows 10 (1903 and up)`
    - ![ウィンドウキャプチャ (設定)](jp/window_capture-needle_jp.png)
4. After selecting `needle` in `Sources`, select `filter`.
    - ![フィルタ](jp/needle_filter_jp.png)
5. Add `Color Key` to `Effect Filters` and set the background color to `Key Color Type`.
    - ![フィルタ設定](jp/needle_filtered_jp.png)
6. DONE!
    - ![結果](jp/end_result_jp.png)
