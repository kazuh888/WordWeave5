# WordWeave5を別のWindows PCで使う

2026-10-03時点の導入手順である。対象はビルド済みの `wordweave5.exe` を移して使う場合である。実行時に必要なものは利用機能によって異なる。開発用ツールは最後に分けて記す。

## 1. 本体と学習データを配置する

1. 元PCでWordWeave5を終了し、`target\release\wordweave5.exe` を移行先の任意のフォルダーへコピーする。EXEを再ビルドする必要はない。
2. 既存の教材・学習記録も移す場合は、元PCで `%LOCALAPPDATA%\WordWeave5` フォルダー全体をバックアップしてから移行先へコピーする。アプリを起動したまま保存フォルダーを上書きしない。保存ファイルの役割は [READMEの「保存とバックアップ」](../README.md#保存とバックアップ)を参照する。
3. 移行先でEXEを起動し、既存の教材・記録を確認する。Codexのログイン状態とQwen音声評価のAPIキーは、保存フォルダーのコピーだけで移行したと判断せず、それぞれ設定し直す。

Windowsの読み上げはOSの英語音声、録音はOSの既定マイクを使用する。読み上げる場合は英語音声とスピーカー、録音する場合はマイク・Windowsのマイク使用許可を確認する。これらは別途インストールするコマンドラインツールではない。

## 2. AIチャット・教材生成を使う場合：Codex CLI

1. 移行先のWindows利用者で [OpenAI公式のCodex CLI導入案内](https://learn.chatgpt.com/docs/codex/cli) に従い、**Windowsで起動できるCodex CLI**を導入する。ChatGPTのデスクトップアプリを置くだけではCLIの所在を確認できない。
2. 新しく開いたPowerShellで `Get-Command codex` と `codex --version` を実行し、起動できることを確認する。見つからない場合はCodex CLIの実行ファイルの所在とPATHを確認する。
3. `codex login` を実行し、このPCでWordWeave5を使うWindows利用者としてChatGPT認証を済ませる。APIキー認証はWordWeave5のCodex経路では使わない。
4. WordWeave5の「設定」→「接続・ChatGPT認証を確認」を実行する。設定の実行ファイル欄は `codex` で自動探索できる。明示した実行ファイルを使う場合は、実在する `codex.exe` または `codex.cmd` のパスを指定する。

Voltaの有無と、Node.js/npmが必要かどうかは別の判断である。

| Codex CLIの導入方式 | Node.js/npm | Volta |
| --- | --- | --- |
| 公式Windows用CLIを直接導入 | WordWeave5からCLIを使うためには不要 | 不要 |
| npmで導入（Voltaなし） | Node.jsとnpmが必要。通常のnpmランチャーは起動時にもNode.jsを使う | 不要 |
| VoltaでNode.js/Codexを管理 | Volta側でNode.js/npmとCodexを用意する | 必要 |

npm方式を選ぶ場合は、移行先でNode.js/npmを導入してから公式手順に従ってCodexを導入し、`node --version`、`npm --version`、`codex --version`を確認する。Voltaを使わないからNode.js/npmが不要になるわけではない。

Voltaが作った `codex.cmd` を使用する場合は、同じPCに `volta.exe` と管理対象のNode.js/Codexも必要である。WordWeave5は既知のVolta shimを検出し、Volta本体を通してCodexを起動する。一般の `.cmd` を使う場合はWindows標準の `cmd.exe` を使う。[Codex CLI公式案内](https://learn.chatgpt.com/docs/codex/cli)・[VoltaのWindows導入](https://docs.volta.sh/guide/getting-started)。

## 3. 「読んで発音を確認」でMP3などを使う場合：FFmpegとffprobe

対象の操作は、**「教材」→「主な例文」→「読んで発音を確認」→「音声を選ぶ」**である。ここでMP3・AAC・AMR・3GP・3GPPを選ぶと、ファイルの検査と「原音を聞く」ための復号に **`ffmpeg.exe` と `ffprobe.exe` の両方**を使う。送信には選択した元ファイルの内容を使い、再生用に変換した内容へ置き換えない。

通常の学習、Windowsの読み上げ、この画面でのマイク録音、WAV（PCM16）の選択・再生だけなら両ツールは不要である。

1. [FFmpegのダウンロード案内](https://ffmpeg.org/download.html)に掲載されたWindows用ビルドの配布先から、必要なdecoderを含むビルドを入手する。FFmpegプロジェクト自身はソースを配布し、Windows実行ファイルの提供元は同ページから案内される。配布元・ライセンスも確認する。
2. 展開した `ffmpeg.exe` と `ffprobe.exe` を、WordWeave5のEXEと同じフォルダーの `media-tools` サブフォルダーへ置く。選んだビルドに必要なDLLがあれば配布元の指示に従って同梱する。

   ```text
   WordWeave5\
     wordweave5.exe
     media-tools\
       ffmpeg.exe
       ffprobe.exe
   ```

3. そのフォルダーでPowerShellを開き、次を実行する。

   ```powershell
   .\media-tools\ffmpeg.exe -version
   .\media-tools\ffprobe.exe -version
   .\media-tools\ffmpeg.exe -hide_banner -decoders | Select-String libopencore_amrwb
   ```

   最初の2行でそれぞれ版情報が表示されれば、両実行ファイルの起動を確認できる。AMR-WB形式も使う場合は3行目に `libopencore_amrwb` が表示されるビルドが必要である。表示されなければ、そのビルドではAMR-WBを選ばない。

同じフォルダーに配置できない場合は、両EXEが入った**同一の絶対パスのフォルダー**をWindowsのPATHに登録し、WordWeave5を起動し直す。アプリはまず自分の隣の `media-tools`、次にPATH上の絶対パスのフォルダーを探索する。FFmpegを自動ダウンロードしない。対応する音声の中身と上限は [共通音声ツールの説明](../tools/qwen-audio/README.md#音読を評価する)を参照する。

## 4. Qwen音読評価を使う場合：接続設定

1. WordWeave5の「設定 → AI接続 → 音読評価：Qwen」で「Qwen接続設定を編集」を開く。
2. Alibaba Cloud Model Studioで契約したリージョンを選び、そのWorkspaceのAPI HostとAPIキーを入力する。東京・シンガポール・北京・香港・フランクフルト・バージニアに対応する。対象モデルの利用権とWorkspaceの提供範囲は管理画面で確認する。
3. 同じページ内の「Qwen接続を保存」で保存する。この専用操作だけがWindowsの資格情報を更新する。通常設定の保存ボタンは使わない。取り消す場合は「Qwen編集をキャンセル」を選ぶ。保存だけではAPIを呼ばない。

編集中の「接続を確認」では、その入力値でモデル一覧を取得する。音声や例文は送らず、自動保存もしない。成功しても音声評価の実行権限・品質・費用は未確認である。接続先が一覧APIに未対応の場合、別ホストへ自動切替しない。

音読評価はCodex CLIとは別の接続である。キーはWindowsの資格情報に保存され、学習記録ファイルをコピーしても自動的には移らない。地域やWorkspaceを変更する場合は、その接続先のキーを入力し直す。アカウント/サービス利用とネットワーク接続が必要であり、送信すると従量課金が生じ得る。手順と音声条件は [Qwen音声ツールの説明](../tools/qwen-audio/README.md#初回の設定)を参照する。

## 5. ソースからEXEを作る場合だけ

`build.cmd` または `cargo build` を使うPCには、RustのMSVC版、Visual Studioの「C++によるデスクトップ開発」とWindows SDKが必要である。これは**ビルド用**であり、ビルド済みEXEをコピーして使うPCにCargo、Rust、Visual Studio、Git、Pythonを追加する手順ではない。コマンドは [READMEの「ビルド・起動」](../README.md#ビルド起動)を参照する。
