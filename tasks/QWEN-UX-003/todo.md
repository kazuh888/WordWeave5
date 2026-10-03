# QWEN-UX-003 作業リスト

状態：実装・自動検証・合成表示・release生成完了。全体fmt gateと未実施native項目は未完了である。以下は計画担当のP1チェックリストを保持し、実績の正本は末尾の終了記録と[結果](results.md)とする。既存タスクの未完了項目は保持する。

## T0：計画を確定する

- [x] 規約・計画skill・KPIを読み、対象コードと既存仕様からP0を作成する。
- [x] 親がP0の範囲・明示model/effort・所有を確認する。
- [x] Astra/highの外部仕様担当が `spec.md` を作成し、親が全文承認した。
- [ ] Astra/highの独立reviewerが仕様を確認する。既承認要求の再承認roundは不要である。
- [x] Sol/highのdesigner（親）が `design.md` に自動prepare、sent_previewの寿命、上部固定概要を記し、承認済み仕様と照合した。
- [ ] Astra/high reviewerが設計を独立確認する。
- [x] 計画担当がspec/design全文を照合し、Q3-AC追跡・所有・依存・未決をP1へ更新した。

受入：P0の未決4点は解決した。通信/取消し/データ保持の境界を維持する。検証：要求IDと仕様/設計の追跡、リンク、役割model/effortの対応。独立レビューは別gateである。依存なし。規模S、計画担当所有はplan/todoのみ。

## U1：3段階の音読評価を実現する

説明：音声準備から再練習までを一画面で理解できるようにする。対象は本体 `src/app/qwen_reading_ui.rs` と独立テスト `src/app/qwen_reading_tests.rs` である。

受入条件：

- [ ] UX3-AC-01/03/04：3段階、説明配置、丁寧な句点改行、完了後概要の再展開、固定終了が成立する。
- [ ] UX3-AC-02/05：自動確認まで通信0、明示送信のみ1回、旧snapshot拒否、取消し優先、raw/教材の安全性を維持する。
- [ ] UX3-AC-06：標準/狭幅/拡大の表示と操作到達を確認し、未実施の実機/API項目を分離する。

実施・検証：

- [ ] Sol/high test authorが独立回帰を追加する。既存競合試験の期待値は保持する。
- [ ] 変更前RED全件未取得を明記し、新規の状態安全回帰の変更前/後または同等証拠を残す。
- [ ] Sol/high implementer（親）がUIとactionの担当範囲を変更し、焦点回帰 `cargo test --locked --bin wordweave5 qwen_reading_tests` を通す。
- [ ] 自動確認0送信/再描画ID安定、旧ID拒否、上部全段階概要、sent_preview再展開/寿命、段階別error/noticeを合成fixtureで確認する。

依存：T0のP1。規模S（本体1＋テスト1）、難易度中・リスク高。designer Sol/high、implementer Sol/high、test author Sol/high、test runner Sol/medium、reviewer Astra/high。選定理由はplanのU1表に従う。controller・認証・通信providerへの変更が必要なら範囲を親へ戻す。

## Checkpoint：固定差分を検証・独立確認する

- [ ] Sol/medium runnerが対象差分を固定し、`cargo fmt --all -- --check` を確認する。
- [ ] `cargo test --all-targets --locked` を実行して件数/exit codeを記録する。
- [ ] `cargo build --release --locked --bin wordweave5` を実行してexit codeを記録する。
- [ ] 合成native画面で初期/確認/評価中/正常/不一致/評価不能/失敗/取消し、標準・820×650・80/100/125/160%を確認する。キーボード到達を別記する。
- [ ] Astra/high reviewerがread-onlyで仕様・設計・最終差分・新しい検証証拠を確認する。指摘修正は各所有者へ戻す。
- [ ] 関連修正があれば影響する検証だけ再実行する。旧ログを新差分の合格証拠へ流用しない。
- [ ] 親が結果/未検証/利用者受入残件、current索引、取得可能なKPIを記録する。計画担当は他の文書を編集しない。

完了はUX3-AC-01～06と継承する安全gateを満たした時点である。実API・実キー・実学習データの試験と公開はこのリストに含めない。

## 終了記録（親による事実統合）

- [x] 仕様・設計をAstra/high reviewerが独立確認した。
- [x] 3段階表示、説明配置、丁寧語・句点改行、折畳み再展開、固定終了を実装・確認した。
- [x] 自動確認0通信・明示送信1回・旧ID拒否・取消し優先・receipt寿命を独立回帰で確認した。変更前RED全件は未取得、初回9失敗と対策後全465件成功のログを保持した。
- [x] 音読34件を含む全体465件、debug/release build成功。重複した焦点コマンドは実行せず全体内の同試験結果を使った。
- [x] 標準8状態と最小幅80/100/125/160%のnative14画像を親・reviewerが実見した。新規P1/P2なし、関連修正後の新しい証拠を使用した。
- [x] 結果・current・KPIを更新し、EXE現物hashを独立照合した。
- [ ] 全体fmt gate合格。既存39ファイル差分でexit 1、対象3ファイルは整形成功。無関係な編集は保持し、gateを免除していない。
- [ ] 取消し終端のnative画像とnativeキー操作。headlessの実Tab/Enter/Space・取消し試験で代替合格とはしない。

利用者受入・実マイク/API・DPI等は別途であり、全gate合格・最終受入済みとは判定しない。[検証](runner.md)・[native](native.md)・[独立レビュー](review.md)を参照する。
