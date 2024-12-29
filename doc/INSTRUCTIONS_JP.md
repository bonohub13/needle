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
1. OBS Studioとneedleを起動します。
2. `ソースを作成/選択` 下の `ウィンドウキャプチャ`を押します。
    - ![ウィンドウキャプチャ](resources/jp/window_capture_jp.png)
3. needleをソースとして選択後、 `キャプチャ方法` を `Windows 10 (1903以降)`に指定します。
    - ![ウィンドウキャプチャ (設定)](resources/jp/window_capture-needle_jp.png)
4. ソース内の`needle` を選択後、`フィルタ`ボタンを押します。
    - ![フィルタ](resources/jp/needle_filter_jp.png)
5. `エフェクトフィルタ`内の`クロマキー`フィルタを押します。
    - ![フィルタ設定](resources/jp/needle_filtered_jp.png)
6. 背景色を`色キーの種類`に指定して、`閉じる`を押してた後に背景色が透明化されていることを確認できます。
    - ![結果](resources/jp/end_result.png)
