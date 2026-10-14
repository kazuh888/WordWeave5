# ET3 試用EXE提供・独立レビュー記録

## ET3-FX1 Pre：PASS（2026-10-14）

独立reviewer `trial_delivery_review`（指定 `gpt-6-astra/high`、read-only）は、all-targets compile blockerを是正する限定設計をPASSと判定した。確定欠陥/受入阻止/要回答質問/任意提案0。実装・compile・runtime・G0解除・提供の成功を意味しない。

入力HEAD `d446c5dd32177815ee322a39cf51ab84a99ee458`、計画SHA `4DACB561260B32D15570C07D7F202E82B0D6719149D1EDE194CB0057DA61100F`、設計SHA `B2FEFA5C9E1B10F509939F5235C2B45DBFA58F6455B83C58F19D6298F8422969`、契約SHA `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D` を現物照合した。source-beforeの168項目/作者4markerは不変、新helper不存在、dirtyは文書のみ、diff検査成功。

確認したのは、ai/mediaのファイル直下private Windows/test限定support moduleによるsibling path解決、旧media再公開、stateless helperの同体抽出と明示imports、非Windows/非testでの除外である。既存7consumerと通常bin23/試用bin33/integration各11の予定集合、A08/A09/B/Dのguard・worker終了契約、import逆置換によるai oracle保持と他3marker一致要求を照合した。隔離のAdminFalse/1exact1child/合成環境/Hidden/45秒/killwait/子回収後cleanup/完了markerは変更しない。

修正は作者がsrc/ai.rs・src/media.rs・新src/early_trial_test_support.rsのtest/fixtureだけを担当する。独立runnerの両構成all-targets no-run、新helper込み9source＋integration/Cargoの固定、構成EXE別の親G0、集合不減と独立Postを必須とする。既定ET3の全体/関連crate/release/native/提供gateは維持する。source本文不一致や所有不足では停止する。

未検証は実差分の同体性、Rust compile、新EXE listと動的結果である。reviewerの編集/build/runtime/再委譲/公開0。週枠61→61%（共有）、終了20:30:43 JST・最終応答前、counter/session/provider返却値/親子包含は未取得。

## FX1親sourceG0・両構成no-run限定解除

作者は3sourceだけを修正し所有返却した。親がai/mediaの実diff全体と新helper全文を読み、module宣言/ai import/旧media再公開/同体抽出だけであることを確認した。通信GLOBAL・録音global・実入口遮断/加算/poison/Dropとworker・guard寿命は変更していない。新helperは明示importsと元doc/signature/bodyだけで、状態やtest caseを追加しない。

親がGitの修正前mediaから関数を再抽出し、設計どおり行頭4space/CRLF→LFだけを正規化して新関数全文と比較した。一致true、正規化SHA `2C5A41C49D78717C41A8118738F5F8ACFEF73FB1121D90C95C62F9E357A17B0F`。作者evidence SHA `D5357C2602B17962C6A94520B13B36D37EF80687E1155BE722AB38E37E18C563`、afterSnapshot全12項目（新helper込み9source＋integration＋Cargo2）も親現物hashに全一致した。作者証拠のai import逆置換marker一致/他3marker一致を照合、旧release SHA不変も再確認した。

| 変更source | 固定SHA256 |
| --- | --- |
| src/ai.rs | `94B85460696455D8D9D48593B0C299B5E849B10A45C2039E43F18D057E062A99` |
| src/media.rs | `A3EC60D795C63052E6E3EEAC45EE7903CAAD97752242C005DBFE801EE6DF2673` |
| src/early_trial_test_support.rs | `40A43200CE106B555E4688716179BA88C972D104AF401F059A86A15FF33FE34C` |

この固定組に対して独立runnerの通常/試用all-targets no-runだけを解除する。各構成bin/codex_processを含む全artifact JSONと前後source一致、profile/features/絶対EXE/SHAを返すまでlist/runtime禁止。旧失敗時部分artifact、旧ET2、bin→integrationの解除を流用しない。source/EXE変更・errorでは停止、作者以外のsource修正不可。関連crate/release/native/提供は後続のままとする。

## FX1独立静的確認：指摘0、Post保留

同reviewerは実差分を独立確認し、承認済み3file境界、helper同体SHA、ai markerのimport逆置換による旧hash一致、他3marker不変、修正後12項目と他166項目の現物一致を確認した。確定指摘/要回答質問0である。compile・新EXE list・動的結果は未確認であり、最終Post/Closedではない。編集/build/runtime/再委譲0。共有週枠62→62%、終了20:39:14 JST・最終応答前。指定Astra/high、返却metadata/token/session/親子包含は未取得である。

## ET3 native提供条件の限定確認

同reviewerの限定読取判断では、契約ET-03/設計/計画が要求する合成native導線とrfd起動拒否dialogの確認は提供前gateである。特にOS dialogのWindows DPI 100/150/200%をegui zoomや正常起動だけで代用しない。実credential/API/micを利用した検証を要求するものでもない。mock画面の直開きだけでは既存音読導線の到達証拠にならない。IME・実Codex/Qwen・mic・TTSは既存どおり利用者実機受入へ残せるが、native gateの暗黙免除にはならない。

親はDPI 150/200%の確認だけを利用者試用時確認へ移す承認を非同期で求めた。回答待ちであり、既定gateは未変更である。他の検証は継続する。限定判断の編集/実行0、共有週枠62→63%、終了20:44:17 JST、返却metadata等の欠測は継続する。

## FX1新EXE Gate：両構成のlist・限定回帰を解除

独立runnerの通常/試用all-targets no-runは各exit0/build-finished=true、各24artifact、compiler error0。親が固定12項目の現物hash差0と旧release不変を再確認し、次の4EXEをartifact JSONおよび現物hashへ照合した。binとintegrationを区別し、通常features=[]、試用features=[early-trial]、いずれもprofile.test=trueである。

| 構成/target | 絶対EXE（repo rootからの固定相対表記） | SHA256 |
| --- | --- | --- |
| 通常/bin | `target/early-trial-build/default/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `CDE160AEB81AE335379CE624E9EC942A28FDD21766823685F528DF5A6FE508F5` |
| 通常/codex_process | `target/early-trial-build/default/debug/deps/codex_process-b80f5fbdc29c53b8.exe` | `54075C6EEDA79877A75687EEE6BEDDF7BF5D819CE261A1C0C4A0D12C746B7E14` |
| 試用/bin | `target/early-trial-build/trial/debug/deps/wordweave5-e0ab9f88db3adadd.exe` | `3014F3091436122D86DDC3543D5F842D853AD5CC779B651A4D9A145EACCBD107` |
| 試用/codex_process | `target/early-trial-build/trial/debug/deps/codex_process-4e6a5997dab868a1.exe` | `C70CE963CAB94990CD48599366CFAA3F71B0C41E3F1EE7642DED190579B0F625` |

絶対基点は `F:/Kazuhiro/GitHub/WordWeave5/`。この4EXEだけlistとFX1限定回帰を解除する。録音安全3件を先行し、P01安全順序、helper consumer集合不減、integration自身のA群11件を確認する。各起動で非管理者確認・EXE hash・合成環境・Hidden・期限・回収・該当child完了markerを維持する。source/EXE不一致や未知の失敗で停止する。他artifact、全体実行、related/release/native/提供は未解除である。

## FX1最終Post：PASS / Closed

独立reviewerは両no-run raw JSON、4EXE現物、全88個別result/stdout/stderr、集計JSON/CSVを照合した。通常bin26/integration11、試用bin36/integration11の84/84が実名付きexact1件PASS、ignored0。helper78件の実名child marker/AdminFalse/exit0、非helper6件の外側結果、4listの集合不減、実行順、期限/Hidden/wait/fixture残存0、終了後対象process残存0を確認した。

build前後/runtime後170項目と現物差0、G0の12項目/4marker/Cargo設定/旧release不変、wrapper hash一致。既確認の同体抽出とoracle/global/worker寿命保持の静的判定、同snapshotのfmt/diff成功を接合した。FX-R1–3の限定是正をClosedとし、確定指摘/要回答質問0。ET3全体/related/release/native/提供は閉じない。reviewer編集/実行0、終了20:59:39 JST・最終応答前、共有週枠63→64%、返却metadata/token等は未取得である。

## 既定ET3全体gateへ復帰

親は通常/試用それぞれ全24artifactの現在SHAを保存JSONへ全照合した（各不一致0、test profile各22、features通常[]/試用[early-trial]）。固定source不変とFX1独立Postを受領し、この組で既定の通常/試用all-targets runtimeを解除する。各構成の既定Cargoコマンドを省略せず、合成環境・非管理者・有限timeout・終了/残存確認を維持する。full run中の非test artifactを起動する許可ではない。新source/EXE変化、未説明の失敗では停止する。

関連crateの既定no-runと通常/試用/単独qwen-audioの別target release buildも許可する（buildのみ）。関連test EXEの対応照合までruntime不可、releaseのnative起動/最終提供も未解除である。native条件の未回答承認を推定で埋めず、旧製品releaseを上書きしない。

### 通常full開始時の再compileによる解除失効

既定Cargo full commandは固定sourceのまま再compileを開始した。原因を環境差と断定せず、親は所有する検証Cargoだけの停止/回収を指示した。runnerは所有treeを親子/時刻/commandで照合し、21:08:03 JSTにkill/wait、対象残存0・ユーザーprocess停止0を確認した。wrapper exit1/Cargo exit-1は意図した停止であり、製品テストFAILとして数えない。raw test result/Running/Finished行の観測0、実行成功証拠へ流用しない。旧通常24artifactは23件が変更又は不存在となったため、旧通常EXEの実行解除は失効する。source170項目差0、旧製品/単独tool EXE不変である。

次は再compileの限定診断と、fullと同一process環境・同一合成rootでのno-run固定→親の新artifact照合である。現時点、再build/runtime/related/releaseはrunnerへ停止返却し、native/提供も未解除のままとする。FX1の既取得・独立確認済み84件は当時のsnapshot証拠として保持し、新EXEの成功へ読み替えない。

### 再compileの局所原因と許可待ち

runnerのfingerprint診断は複数依存crateについて `MissingFile { path: "C:\\Users\\安井" }` を明示した。対応Rustc dep-infoではプロフィール名のU+3000が無escape、Cargo binary dep-infoには切れた不存在prefix1件と正しいfull path11件が共存する。固定環境の通常no-runで再生成したdep-infoにも同entryが残り、既存試用側にも同じ問題を確認した（`full2-both-dep-info-prefix-audit.json`）。少なくとも同一環境だけでは次の再buildを防げないことが分かった。Cargo全般/全日本語pathへの一般化はしない。source、主要fingerprint、toolchain/configの不一致やGit hookは検出していない。

親はprocess限定Cargo target runnerによる「build後・実行前の新EXE照合」の一時ラッパー導入許可を利用者へ質問した。公式[Cargo runner仕様](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner)で機構の存在は確認したが、適用設計・実装・承認は別であり未着手。source/依存/fingerprint/OS/global Cargo設定の書換えや、テスト/合否基準の緩和はしない。原因既知の試用no-runの無用な反復は行わず、進行中の通常no-runだけ安全に終了して返却する。

承認待ちの間も未影響の既定release build（試用→通常→別target単独qwen）だけは追加解除した。新EXE/構成/hash/source保全を返すまで起動不可、全体runtime/native/提供は依然未解除である。build成功を全体テスト成功の代用にしない。

親の代替案read-only確認では、既存`.cargo`/`.rustup`の8.3短縮pathにもユーザー名部分のU+3000が残った。短縮名へのprocess環境切替だけではこの空白を回避できないため採用していない。alias作成/移動/環境変更0である。

### 配布用buildの進捗（提供判定ではない）

試用releaseはexit0/build-finished=true、features=[early-trial]/opt3/debuginfo0/debug_assertions=false/overflow_checks=false/test=falseである。`full2-trial-release/artifacts-fixed.json` のEXE SHA `E3A0A87861D835C1E142072A14F3EBF7CF9DA0A8B95D404F4BF99402D4B1BBD2` を親が現物へ照合した。旧通常製品SHAは保全値一致。source170/G0不変・非管理者/Hidden/有限期限/wait/TEMP残存0はrunner記録を参照する。試用EXEのnative起動・提供用copyは0、全体gate成功を意味しない。

通常releaseもrunnerからbuild成功が返った（EXE SHA `661156D57067404BE8A5C232CED6A19442BF5D3F6487EF4B905AF4848AE4F2C2`、features=[]）。単独qwenの別target buildを継続している。全EXEの最終親照合と提供Postは未了である。

### 今回の区切り：3build成功・承認待ち（ET3未完了）

単独qwenもexit0で完了した。親は通常/単独qwenのartifact JSONと現物SHAを照合し、単独SHA `DD93326B164661A74CDF498852DF19DF471C7A3AFBED65980D5AAA45A30526AA`、features=[default,gui,mcp,windows-credentials,windows-job]を確認した。試用と合わせ3release build成功であり、製品test成功件数へ数えない。親がG0全12項目差0、旧通常EXE保全SHA、旧単独 `target/qwen-audio/release/qwen-audio.exe` SHA `57A3BE5B10F117E146E1C8AAC77616AA4A0F0873D74D68D858BD8C6E9A64506E` 不変も再確認した。

runnerの最終source170/marker4/新通常debug24artifact差0、全所有process残存0/wait済み、実API/credential/mic/native/提供copy0は `full2-final-preservation.json` / `full2-final-process-audit.json` と報告末尾へ固定した。runnerの許可作業は終了して所有返却済み、背景で続行するprocessはない。

一時Cargo runner導入とDPI150/200確認の移管は未回答であるため、ここで承認待ちにする。残りは承認範囲の計画/設計と検証手順整備→通常/試用full/関連crate gate→native確認→最終提供Post→ignored提供folderへEXE/使用説明/manifestのcopyである。release buildとFX1の84PASSだけでET3をClosed又は提供済みとしない。実API/実マイク等の利用者受入は引き続き別項目である。
