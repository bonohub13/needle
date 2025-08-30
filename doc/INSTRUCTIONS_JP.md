# 使用方法
1. [OBS Studio + needle](#needle+OBS_Studio)
2. [Windows上の背景透明度設定 (NVIDIA GPUユーザ向け)](#BackgroundTransparency)

## 1. OBS Studio内でneedleの使用方法<a name="needle+OBS_Studio"></a>
### 1.0. 準備 (任意)
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

### 1.1. OBS Studio内の使用例
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

## 2. Windows上の背景透明度設定 (NVIDIA GPUユーザ向け)<a name="BackgroundTransparency"></a>
### 2.0. 前提条件
1. needleの背景色の透明度 (alpha) を1.0未満に設定したこと
2. NVIDIAのGPUドライバをインストール済みであること

## 2.1. Nvidia ドライバの設定
1. `NVIDIA Control Panel`を開きます
    - ![NVIDIA Control Panel](resources/common/2-1-1_NvidiaControlPanel.png)
2. `3D設定の管理`ページに移動します
    - ![Manage 3D Resources](resources/jp/2-1-2_Manage3dResources_JP.png)
3. `Vulken/OpenGlの既存の方法`を`ネイティブを優先する`に設定
    - ![Vulken/OpenGl present method](resources/jp/2-1-3_RenderMethod_JP.png)
