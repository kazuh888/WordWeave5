# I4 core実装引継ぎ

2026-10-04 / ACOUSTIC-HISTORY-001 / implementer。planner指定 `gpt-6.1-sol` / `high`。実行metadata、開始・終了使用量カウンターは未取得である。完了境界は承認済みI4 core実装と独立32件のfocused検証であり、最終検証はrunner、最終受入は利用者に残る。

AGENTS.md、development-team.md、KPI、wordweave-change、指定されたTDDとincremental-implementationを読了した。入力はdesign-i0.md 3節、design.md 7節、core-tests.mdと独立tests/core.rsである。REDはauthorが取得した32件失敗を利用した。再委任、モデル/gate変更、commit/push、実学習データ、外部APIを使用していない。

## 所有と動作

変更した本体は `tools/acoustic-analysis/src/lib.rs`、`profile.rs`、`measure.rs`、`validate.rs` の4本である。manifest/lock/test/他者編集は変更していない。親のReport入力metadata追補を保持した。

PCM非空・1/2ch・8–48kHz・ch割切り・60秒以下を検証し、元borrowを保持する。monoはPCM16/32768の算術平均、DC除去なし。全体RMS、sample丸め25ms/10msの中心時刻、末尾窓、非重複hopの低エネルギー候補と200ms以上の連結区間を実装した。ゼロはDigitalSilenceである。

F0はfake/Missing backendだけで検証した。60ms未満はbackendを呼ばずTooShort。backendからのUnavailable/Failed/InvalidOutputだけをF0欠測Partialへ変換する。InvalidInput/UnsupportedProfile/Cancelled/TimedOut/ResourceLimitは全体Errorである。原音先頭基準の時刻、増加、50–800Hz、無声だけの欠測variantを検査し、線形補間の中央値/P10/P90と全frame数を分母とする割合を算出する。60ms以上の成功backendが空trackを返すケースは親がInvalidOutput→Partialを承認した。全無声のframe列はUnvoicedと割合0で保持する。

共通deadlineとtokenをRust段階・backend前後・公開前で照合する。上限は各時系列6001frame、core buffer合計64MiBであり、指定limitsが小さければ小さい値を使う。mono、RMS、区間、返却F0 Vec capacity、集計scratch、quality、engine/report String、report検証scratchを予算へ計上する。予算不足は切詰めずResourceLimitである。汎用backendの中を強制中断する責任はtraitだけでは実現できず、I5 adapterのprocess制御に残る。

Report.validateはschema/profile、入力metadataとduration、dBFS/Hz/割合、sample丸めRMS時刻/count、pitch時刻とvariant、RMS列から再構成した低区間/合計/割合、F0集計とComplete/Partialを検証する。serde decodeだけを保存受理に使わない。

## focused結果

- PCM境界2件、profile固定1件、ゼロRMS1件、F0分位数1件、resource1件、取消し/deadline3件のfocused実行は成功した。
- 初回全件は31成功/1失敗である。手書きvalid DTOのvalidateは成功したが、serde_json往復の厳密一致がf64末尾差で失敗した。親が所有manifestのserde_jsonへfloat_roundtripを追加し、期待値/APIを維持して解消した。[初回ログ](artifacts/core-green-first.log)。
- 最終 `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --locked --offline` は独立32件成功、0失敗/0ignored、unit/docは各0件、exit 0である。[最終ログ](artifacts/core-green-final.log)。
- `cargo clippy --manifest-path tools/acoustic-analysis/Cargo.toml --lib --locked --offline -- -D warnings` はexit 0である。初回の等価整数境界表現1件を修正した。[clippyログ](artifacts/core-clippy.log)。
- 割当4本の `rustfmt --edition 2021 --check`、`git diff --check` はexit 0。TODO/unsafe/filesystem/HTTP/Qwen接続は割当本体に存在しない。

## 残る境界

I4の32件に未達はない。Praat実process、精度、Windows resource制御・子回収、license、二host永続化、real Qwen、native UIは未検証である。root/qwenへの接続、最終全体Cargo/release、独立review、利用者の受入は親と後続役に残る。これらを32件の成功で代替しない。

標準execはdeny-read ACL適用エラーで起動不能であった。親指示どおりrequire_escalatedを用い、割当範囲の読み取り・整形・合成テスト・静的検査を実行した。自動承認レビューによる拒否は発生していない。
