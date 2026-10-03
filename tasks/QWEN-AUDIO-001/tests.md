# QWEN-AUDIO-001 独立テスト作成

担当 `/root/qwen_tests`。planner指定 `gpt-6.1-sol / high`、実行metadata・開始/終了token counter・親子包含は未取得である。所有は `tools/qwen-audio/tests/**` と本書のみ。AGENTS、development-team、KPI、plan/spec/design/sources、`agent-skills:test-driven-development` 全文を読了した。production・manifest・lock・他者差分を編集しない。

完了境界は承認済み受入条件からの独立テストと隔離fixture、焦点実行、実装者/runnerへの期待値handoffである。親dispatchにより、採用済みP01/P02と安定したU1公開APIから先行テスト作成を開始した。U0独立レビューPASSED・確定design hash `D4656D44…` の親通知を受けU2/U3を追加した。実API・利用者録音・学習データ・資格情報・ネットワークを使用しない。

## U1

`tests/audio.rs` はQA-AC-005/006/008/014/017の入力部分を扱う。合成RIFFは `tests/support/mod.rs` で生成する。通常mono/stereo・無音・1フレーム、8/48kHz境界、60秒と1フレーム超過/61秒、60秒48kHzのmono/stereoサイズ、全体6MiBと1byte超過、空/偽拡張子、RIFF宣言長、chunk overflow/短いheader/odd padding/未知chunk、fmt/data欠落・重複、fmt長/data切断/frame alignment、byte rate/block align、unsupported形式/深度/channels/RF64、10,000 Unicode scalar/10,001/空白、原ファイル変更/削除後snapshot、read-only保存とpath非漏出を検証する。

fixtureは所有する一時directoryに生成し、そのdirectoryだけをDropで削除する。確定音声は同じbytesかを比較し、実装の結果をgolden値として採用しない。入力検証はproviderへ依存しないため「不正入力→0通信」のcontroller結合証拠は後続テストで採取する。

焦点コマンド（repo root）：

```text
cargo test --manifest-path tools/qwen-audio/Cargo.toml --test audio --locked --target-dir target/qwen-audio
```

U1先行時の実行：上記コマンドはexit 1、`manifest path tools/qwen-audio/Cargo.toml does not exist` である。当時23個の受入テストを作成し、直接rustfmt（edition 2021）が成功した。production package/manifestが未作成であり、この時点の挙動REDは未取得である。manifest不在/compile failureを挙動不一致とは扱わない。本タスクは新機能であり、報告済み既存bugの実挙動再現は該当しない。後続runnerが固定差分の新しい実行証拠を記録する。

U1に必要な公開APIは設計通り `AudioInput::{parse,info,bytes}`/Clone、`ReferenceText::{new,as_str}`、`read_wav`、`SafeError::{code,disposition}`/Display/Debug、`ErrorCode`/`SendDisposition`である。追加dev-dependencyは不要である。

## U2/U3（確定設計から独立作成）

| ファイル / 件数 | 受入ID | 独立した期待値 |
| --- | --- | --- |
| provider.rs / 12 | 004/008/009/010/012/013/014/022 | 要求モデル/effort/max_tokens、確認済み音声/参照の一致、要求回数1、UTF-8/CRLFの全byte分割、reasoning非混入、実値欠測None、usage未取得値を合算しない、HTTP/接続断/timeout分類、限度、stopとDONE両必須、DONEでEOF待たず成功、秘密反射のdecoded比較、quote/backslash偽key、取消0/1送信 |
| feedback.rs / 7 | 009/010 | assessment整合、heard_text非補完、nullも必須、unknown/type/enum/code fence拒否、scalar/array限度、確認英文の部分文字列、サイズ |
| credentials.rs / 4 | 002/004/014 | Tokyo origin/fullbase正規化、userinfo/port/query/fragment/encoded/他host拒否、秘密Debug非漏出、固定SafeErrorメッセージのDeserialize改竄拒否 |
| session.rs / 6 | 011/012/019/020/021 | active1、Running前complete拒否、終端first commit wins、get非消費、UUID v4、最新16件FIFO・読取で昇格せず・再起動で消失 |
| consent.rs / 6 | 003/005/006/007/008/012/014 | fake保存失敗/取消で旧設定保持、read失敗でも修復導線、選択/保存/取得0送信、明示send1・二重/編集中差替え拒否、原file改変/削除のsnapshot保存、送信前/後取消 |
| mcp_stdio.rs / 12 | 018/019/020/021/022/023 | 3toolのみ、JSON-RPC構文/ID、未知入力でGUI無し、busy/passive get、cancel先行grantなし、結果再検証とCommitted、旧子回収後new launch、起動/Ready timeoutの枠回復、EOF強制kill/reap、最新16件と再起動 |
| gui_geometry.rs / 2 | 007/016 | pure render、幅800/360・zoom1/1.5/2、日本語Send glyphの両軸centerと右端、長文/長file/進行/失敗/取消/結果のfinite geometry |

合計72 testsである。共有fixture `support/provider.rs` はHTTP境界だけをfakeにし、`support/mcp.rs` は実Tokio duplex/JSON-RPCとfake GuiLauncher/GuiProcessを使う。資格情報試験は実Credential Managerへ触れない。仮想時間を使い180秒・Ready timeoutを実待機しない。結果のtext/structuredContentの一致も検証する。

```text
cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked --offline --target-dir target/qwen-audio
```

初回追加試験はtokio-util未cache・lock未作成により依存解決でexit 1であった。挙動RED/署名エラーではない。親から実装担当のlock/依存解決完了を受けて再実行した。最初のcompile中に作者のfixture内module pathを `crate::support::wav` へ訂正した。これはfixture修正であり受入期待値の変更ではない。全ファイルrustfmt成功。

実行session `49290` はexit 0、72 passed / 0 failed / 0 ignoredである。内訳はaudio23、consent6、credentials4、feedback7、gui_geometry2、mcp_stdio12、provider12、session6である。公開API署名不一致はなかった。共有fixtureを別suiteへ再利用したdead_code警告のみである。SSE DONE/JSON escape秘密反射の本体修正はコンパイル前に実装担当が適用したため、作者が未修正挙動REDを取得したとは主張しない。初回成功は仕様から独立作成した新機能テストの成功であり、fix前RED証拠ではない。

これは並行実装中の作者焦点検証であり、最終runnerの固定差分・locked test/release・root gateの代替ではない。親へ件数とコマンド/AC/fixture/欠測を共有済みである。担当所有外の修正はゼロ、仕様/quality gate/model変更もゼロ、commit/公開なし。使用量と実行model/effortのcounterは最後まで未取得であり推計しない。

## 最終レビュー修正の独立回帰

親が本体固定・レビュー修正の回帰作成をdispatchした。指定/所有/Skillは前節と同じである。親採用のprotocol metadata検証、Running claimから親180秒監視、pipe EOF以外のprocess終了監視、Win32 blob入力境界を期待値として固定した。

- `tests/internal/credential_record.rs` 4 tests：実装担当が所有するcredentials.rsのcfg(all(test,windows)) path includeからprivate helperを呼ぶ。NULL/0・NULL/正数、所有allocationのnonnull/0、2561byteの実在allocationで拒否、valid1/32/2560byteのcopy一致と元領域zeroize。すべて隔離Vecであり実OS credential呼出しなし、allocation外のlengthを与えない。
- `mcp_stdio.rs` 5 tests追加（計17）：params._meta.progressTokenの文字列/整数がstart/get/cancelで許可、不正metadata/token拒否、tool arguments未知path/key/endpointの従来拒否、AwaitingUser181秒不変・Running179秒維持→180秒超過Timeout Failed MayHaveBeenSent→Cancel+Committed、late成功非採用、completed/cancelledのdeadline非上書き、stdoutを保持するfixtureのprocess終了検知（待機/送信中のdisposition）。
- fixture `support/mcp.rs` のprocess退出通知はstdoutとは別に制御する。EOFだけを原因とする検知と区別する。

追加9 testsにより合計81 testsである。focusedコマンドは `cargo test --manifest-path tools/qwen-audio/Cargo.toml --lib --test mcp_stdio --locked --offline --target-dir target/qwen-audio`。session7476はexit 1、production側の `#[path="../../tests/internal/credential_record.rs"]` が `tools/tests/` を参照するcompile failureである。作者の所有ファイルはtools/qwen-audio/tests/internalに存在する。正しい `../tests/internal/credential_record.rs` へのproduction修正を親へ返した。挙動REDではなく、期待値を緩めない。所有外のproduction変更なし。

親によるinclude pathの1行統合修正後、同じfocusedコマンドsession58442はexit 0、private blob4 + MCP17 =21 passed /0 failed /0 ignoredである。期待値変更はない。新規3ファイルの直接rustfmt --checkもexit 0。全 --all-targets --locked --offline --target-dir target/qwen-audio のsession22016はexit 0、合計81 passed /0 failed /0 ignored（lib4 + integration77）である。修正前の挙動REDは未採取であり、compile接続不一致と分ける。

全package `cargo fmt --manifest-path tools/qwen-audio/Cargo.toml -- --check` はexit 1、唯一の差分はproduction `src/provider.rs:264` の秘密反射iterator整形である。作者はproductionを整形せず親へ差戻した。tests新規ファイルの整形検証は成功済み。単純整形修正だけなら挙動test再実行は不要、package fmt再確認は親/runnerが行う。

MCP fixtureは公開supervisorの状態/pipe/lifecycleの証拠であり、実OSプロセスCLI/Job Object/親crash/native資格情報の受入ではない。GUI geometryはheadless表示の自動証拠であり、日本語font/native DPI/IME/keyboard/screenreader操作は親/runnerの別証拠である。返却結果からのkey非漏出だけではOS/HTTPの全memory消去は証明しない。実API接続・課金・音声評価品質は未検証である。
