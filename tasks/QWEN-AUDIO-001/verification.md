# QWEN-AUDIO-001 テスト実施記録

担当: 独立test runner。計画割当 `gpt-6-luna / medium`。実行model metadata、開始/終了usage counter、親子usage包含関係は未取得であり、推定しない。所有変更は本書と `runner-*.log` のみ。ソース、期待値、settingsは変更していない。

## 対象固定

- HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。変更は未commitのためHEADだけではpackage差分を識別できない。
- `runner-diff-manifest.log` はroot `Cargo.toml`/`Cargo.lock`、全 `tools/qwen-audio/**` の個別SHA-256と統合指紋を記録する。統合SHA-256: `3931AA62B9675C3F9B04B5B24B2C3CE576754E4A297EB5CB342C6AA9A8EDB473`。
- 最終release EXE `target/qwen-audio/release/qwen-audio.exe`: SHA-256 `9B36D56936D39852F0A24C66B153D5BBA475514F8FF65C5DF17F4041DEBB64CC`。
- 初回build時のEXE hash `20241333…E215C3E` は、親によるGUI style修正とfmt後の古い版であり、最終対象ではない。最終buildで再生成した。

## 実行結果

| コマンド | exit | 結果 | 証拠 |
| --- | ---: | --- | --- |
| `cargo test --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --all-targets --locked` | 0 | 10 test groups、81 passed / 0 failed / 0 ignored。group別はunit 4、bin 0、audio 23、consent 6、credentials 4、feedback 7、gui_geometry 2、mcp_stdio 17、provider 12、session 6 | `runner-tests.log` |
| `cargo fmt --manifest-path tools/qwen-audio/Cargo.toml --all -- --check` | 0 | 差分なし | `runner-fmt.log` |
| `cargo build --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --release --locked` | 0 | release EXE生成。unused-mut警告5件、build failureなし | `runner-build.log` |
| Node `spawn`、`stdio:['pipe','pipe','pipe']` によるrelease `--mcp` protocol smoke | 0 | initialize/tools/list/ping/get unknown/cancel unknownを送信。notificationを除く5応答すべてJSON、tools 3件、unknown get/cancelはerror、stderr 0 bytes、stdin EOF後process exit 0 | `runner-cli-smoke.log` |
| release `--ui-fixture result` | 1 | debug専用fixtureを拒否。stderr「現在の状態では操作できない」 | 同上 |

all-targetsの警告は主に共有test fixtureのsuite別dead-codeである。release側には`main.rs`のunused `mut`警告が5件残る。警告を理由としたtest変更・無効化はない。

親が既に実行し、今回root側入力に追加変更がないため再実行していないroot回帰は、`results.md`及び `root-test.log` / `root-build.log` にある。これは今回のrunnerコマンド結果と混同しない。

## native境界・残件

Node smokeは合成JSON-RPC protocolとrelease CLI拒否分岐の証拠であり、API接続・認証・音声評価品質の証拠ではない。親が別途採取した `native-process.log` には実releaseプロセスのGUI子起動、cancel、親EOF/kill後の子回収が記録されており、親のnative画面画像もタスクdirectory内にある。これらは親担当の証拠で、本runnerが独立再実施したものではない。

実資格情報/Credential Managerでの保存復元、Tokyo実API接続、課金発生を伴う評価品質、実Codex環境へのMCP登録は未実施・未受入である。試験中にAPI key、利用者録音、教材、実MCP設定を使用していない。PowerShell直接起動の初回smokeではWindows GUI subsystemのため終了コード/stdioを取得できなかった。その起動方法による観測であり、機能不合格の証拠とはしない。Node明示stdioによる再試験は上記の通り合格した。

## handoff

テストとbuildは固定指紋のpackage内容で成功した。review担当と親の最終受入には、上記fingerprint/EXE hashおよび native証拠の担当境界を引き継ぐ。usage counter欠測は別途親のKPI記録へ伝達する。
