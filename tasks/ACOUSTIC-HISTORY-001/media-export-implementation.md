# I2d-M 媒体export実装

2026-10-12 / ACOUSTIC-HISTORY-001。実装担当の本体接合とfocused検証を完了し、所有を親へ返却する。**独立作者の25件がGREEN、最終runner/Post/利用者受入は未了**である。HEAD `24fa10c` 上の未commit差分であり、再委任・commit・pushはしていない。

入力は [media-export-design.md](media-export-design.md)、[Pre PASS](media-export-review.md)、[独立テストとRED](media-export-tests.md)、親の2file所有指示である。AGENTS、開発7役、KPI、wordweave-changeを適用した。描画・ラベル・レイアウトを変更しないためUI Skillは非該当である。指定model/effortは `gpt-6-astra/high`、実行metadataとusage開始/終了counterは未取得。thread `01a12df0-346e-7f83-b56a-f4734b1c4b5d`、session `01a08550-055c-7aa1-9834-905e4a7ce148`。親子usage包含関係は未確認、集計は親へ渡す。

## 変更と境界

- `src/store.rs:1421` の固定Errを、単一external guard取得、全名形式・ordinal衝突検査、新規directory作成、返却resolved pathを基点としたmember順書込み、completion最後の同期操作へ置換した。既存under_guardを再利用し、guardはcompletionのResult確定までlocalに保持する。指定cfgで作者observerのAfterDirectory/BeforeCompletionを呼ぶ。
- `validate_external_bundle_names` とWindows内 `validate_bundle_leaf` を追加した。単一Normal leaf、空/dot/parent、元文字列の両separatorとtildeを検査してから既存validate_nameへ委譲する。全名検査後に既存same_planned_nameでmember同士/member-completionを比較する。非WindowsはIdentityUnavailableで拒否する。
- `src/app/chat_media.rs:584` の音声と`:597`の原文は既存write_settings_exportへ接合した。`:609`の一式は全bytes/marker生成後に実Storage.dir優先・Noneのみ既存resolverでrootを一度確定し、借用sliceをbundleへ渡す。全経路とも成功の`?`後にだけ対象payload/確認flagをclearする。
- 他媒体、Progress/dirty、write-block、教材承認、既存UIを変更しない。直接atomic_writeの残置、別gate、依存、rollback、retry、production故障設定は追加していない。

最終source SHA256：

| file | SHA256 |
| --- | --- |
| src/store.rs | `12023DF049CFD57C034CD95B87FDF8D20A3368A8A2F24B733BA01457A99338D9` |
| src/app/chat_media.rs | `24171BE283A1D856A2752FA9CEE60D3AE4D43EE4386EFAC6C1B1C5C09C47D5F3` |

作者test moduleは編集せず、作者定義のLF正規化区間hashと一致した。bundle `09FB268E91CBFF4A8D1C7BC80785F5719081D200BCC6FDE97BBBF75C94B44D11`、media `F71AF7A57873928641F8896E58D5801FB0E49F8F70D967E2BEE0A854593D9A5D`。親のcheckpoint完了連絡後に編集し、他者の変更を保持した。

## focused実行証拠

`rustfmt --edition 2021 --check src/store.rs src/app/chat_media.rs` と `git diff --check` PASS。`cargo test --locked --lib --bin wordweave5 --no-run` はexit0、2m21s。logは `logs/media-impl-build.log`。警告を抑制していない。

続く `cargo test --locked --lib store::i2_export_bundle_tests` はlibraryの再compileを伴い、sandboxで4 PASS/5 FAIL、exit101となった。これは通常権限実行と分け、行動REDへ加算しない。logは `logs/media-impl-bundle-sandbox.log`。再compile後にlibrary EXEが変わったため、以前のhashを指定した直接起動はhash不一致で**起動前に停止**した。次の実行では更新済みhashを固定した。不要な再compileが1回生じた点は効率上の手戻りとして記録する。

WindowsIdentity取得成功かつAdministrator=Falseを確認し、true/取得不能なら拒否するguardを用いた。sandbox外の固定EXE実行はHidden/PassThru/Redirect/45秒WaitForExit、所有logs配下の合成TEMPに限定した。RunAs/UAC、実データ、live AI、資格情報は使用していない。

| fixed EXE (target/debug/deps/) | SHA256 | filter / result |
| --- | --- | --- |
| wordweave5-b3388a7bbef44bb4.exe | `55E32499293A8A5BA4A38539128E0CD527ABAC450414843957B4624097294EBA` | store::i2_export_bundle_tests、9 PASS/0 FAIL、exit0、0.76s |
| wordweave5-72e8adffa8f9ef2c.exe | `D8D1D8759DF8621D5E54FE16A0020431A4AE20C4DB9DC7DB13F333BAD8F96C18` | i2_export_media、16 PASS/0 FAIL、exit0、7.00s |

全25実名は `logs/media-impl-{bundle,consumer}-normal-stdout.log` の成功行と作者固定集合に対応する。stderrは同prefixの `-stderr.log`。timeout0、fixture不成立0、合成TEMP残存0を確認した。通常実行用TEMPとhash不一致時の空sandbox準備directoryは空を確認して除去した。本体の追加修正・テスト期待値変更なしでGREENとなった。

## 引継ぎと未検証

本体実装時に発見した設計gapとfocused未達は0。これは最終受入を意味しない。既存attachment/harness/H境界回帰、全体test/release buildは独立runnerへ残す。guard連続生存、binary途中Errからclearに達しない接合、取消し経路の独立静的確認はPostへ残す。native picker/IME/pen、敵対的TOCTOU、OS結果不明時の耐久性を本結果で検証済みとしない。途中出力は設計どおり残り得る。Noneから任意の失敗カスタムrootを復元する能力は追加していない。
