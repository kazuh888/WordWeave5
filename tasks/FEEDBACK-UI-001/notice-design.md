# FEEDBACK-UI-001 U5 通知可読性の内部設計

2026-09-28。入力は親が承認し独立仕様レビューで阻害指摘なしとした [notice-spec.md](notice-spec.md) N01–N08、[notice-plan.md](notice-plan.md) U5a–c、既存 [design.md](design.md) 第5節である。計画者指定は designer `gpt-6-sol / high`。本書は設計担当だけが所有し、本体・テスト・ゲートを変更しない。既存の未コミット変更を基準にする。

## 1. 最小の境界と型契約

REST、永続schema、汎用通知型、外部Markdown依存は追加しない。型付き失敗は `src/material.rs` の検査発生点から `src/ai.rs` の教材専用呼出し、`src/app.rs` の教材結果、`src/app/notifications.rs` の一時表示までに限定する。旧 `Result<_, String>` 呼出元 (`src/store.rs` を含む) は互換wrapperで同じ旧文言を得る。日本語診断文字列を逆解析しない。

`material.rs` に次を公開する。名前と引数は実装・独立テスト双方の契約である。

```rust
#[derive(Clone, Debug)]
pub enum MaterialFailure {
    Other(String),
    Evidence(MaterialDiagnostic),
}
impl MaterialFailure {
    pub fn legacy_message(&self) -> String; // 既存String呼出元向け。旧表示診断を保つ
    pub fn diagnostic(&self) -> Option<&MaterialDiagnostic>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticStage { Generation, SavedDraft }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceCause {
    SourceReference, InvalidPath, MissingItem, EmptyReason, LongReason,
    MissingQuotes, TooManyQuotes, MissingSnapshot, InvalidRole,
    EmptyQuote, LongQuote, QuoteMismatch,
}
#[derive(Clone, Debug)]
pub struct QuoteEvidence {
    pub exchange_index: usize,      // AI応答の0始まり値。表示用の+1はchecked_add
    pub role: String,                // 不正値もそのまま技術情報へ
    pub quote: String,               // AI応答原文全文
    pub original: Option<String>,    // Source.snapshotsの当該話者全文だけ
}
#[derive(Clone, Debug)]
pub struct MaterialDiagnostic {
    pub stage: DiagnosticStage,
    pub cause: EvidenceCause,
    pub cause_detail: String,        // 検査が確定した旧原因。推測用のparse禁止
    pub reason_number: Option<usize>,
    pub path: Option<String>,        // 元path。許可かつ実在時のみ表示名へ変換
    pub target_label: Option<String>,
    pub evidence: Option<QuoteEvidence>,
    pub legacy_text: String,        // 既存API/Storageの表示互換
}
impl Request {
    pub fn build_response_detailed(&self, text: &str)
        -> Result<Draft, MaterialFailure>;
    // 既存 build_response(&str) -> Result<Draft, String> は上記の薄いwrapper
}
impl Draft {
    pub fn validate_evidence_detailed(&self)
        -> Result<(), MaterialFailure>;
    // 既存 validate_evidence() -> Result<(), String> は上記の薄いwrapper
    // 既存 ready() -> Result<Entry, String> の検査・登録可否・順序は維持
}
```

`Other` はJSON形式、Codex/通信、教材の別条件、保存I/O等であり構造化引用通知へ変換しない。`Evidence` は `Source::validate`、`validate_reason`、`validate_quote` に由来する失敗だけである。内部検査は一実装に収束させ、`build_response_detailed` と `validate_evidence_detailed` が呼ぶ。`validate_reason` は path形式→実在項目→理由空白/長さ→引用件数→引用順、`validate_quote` は固定snapshot/role→空白→2000超→原文 `contains` の順を変えない。各分岐で `EvidenceCause` を直接付ける。`Source::validate` の既存 `String` は維持し、詳細経路では `SourceReference` と元の原因文を包む。原因文の部分一致で種別を決めない。旧 `reason_diagnostic`/`source_diagnostic` を `legacy_text` の生成に再利用し、成功判定をUI文言へ依存させない。

`QuoteEvidence` は失敗した引用だけを複製する。引用以前の失敗なら `None`、存在しないsnapshot、不正role、overflowした往復番号なら `original=None` とし、別往復・現在会話で補わない。存在する同じ `Source.snapshots` の同じ往復と正しいroleだけから `original` を作る。全文は一時メモリ上の通知payloadに保持し、`Draft` の永続フィールドへ加えない。既存の `reason_path_label` は、許可pathかつ候補JSONの実在項目を確認した場合にだけ使う。`reason_number` は検査中の `enumerate()+1` であり、複数理由では最初の失敗のみを返す。`Source::validate` が先に失敗した保存案には理由番号や比較本文を捏造しない。

## 2. 非同期・保存案・通知の境界

`Config::material_detailed(Request, Vec<Vec<u8>>) -> Result<Draft, MaterialFailure>` を `src/ai.rs` に足す。Codex呼出しは一回だけで、その一般失敗を `Other`、`Request::build_response_detailed` の結果をそのまま返す。旧 `Config::material(...) -> Result<Draft, String>` は `legacy_message` へ変換するwrapperとする。既存 `Pending.rx: Receiver<Result<AiResult,String>>` は全アプリ共通のため変更しない。教材workerは `material_detailed` の結果を `Ok(AiResult::Material(draft))` または `Ok(AiResult::MaterialFailure(failure))` に包んで送る。後者は教材の業務失敗であり成功結果ではない。一般 `Err(String)` は他作業同様に残す。

`tick` は既存の `pending.kind/key` 一致時だけ結果を消費し、キャンセル・別会話への遷移で破棄された結果から通知を生成しない。`AiResult::MaterialFailure` では既存 `Err` 節と同じ `batch_running=false`/`fetch_then_generate=false` の失敗整理を経て通知へ渡す。`Evidence` だけを `notify_material_diagnostic(MaterialDiagnostic)`、`Other` は従来の `notify_error(String)` とする。旧案 `progress.material_draft`、`deck`、選択/登録state、dirty、保存先は失敗時に触れない。成功時だけ既存の案保存経路を通る。通知用構造は `WordApp` の一時fieldであり `Progress` に入れない。

保存案は `src/app.rs` の既存 `ready(&deck, allow_same_base)` を毎frame実行し、赤い理由と登録disabledを維持する。失敗表示の近くに明示的な「理由を詳しく確認」操作を置く。クリック時のみ同じ案の `validate_evidence_detailed()` を呼ぶ。ただし既存の「対象教材は削除済み」guardが先に失敗する場合はその理由を優先し、引用診断へ分類しない。`Evidence` なら `SavedDraft` 通知を開き、`Other` または根拠検査が成功なら既存 `ready` の理由を一般通知として示す。毎frame自動で通知を再開しない。登録クリックの `ready` 成功後だけ既存の `material_sources.push`→案除去→progress検証→`import_deck` 順へ進む。詳細・閉じる・コピーはこの順序へ入らない。Storage読込の `String` 検査と旧案読込は変更しない。

通知は `MaterialNotice { diagnostic: MaterialDiagnostic, comparison_open: bool, technical_open: bool, raw_mode: bool }` を `src/app/notifications.rs` に置き、`WordApp` が `Option<MaterialNotice>` を保持する。`notify_material_diagnostic(MaterialDiagnostic)` は旧文言を `message`/`notification_error` に設定し、別通知の一時stateを消して三つのUI状態をfalseにする。新規教材通知は同文でも初期化する。`notify_error`/`notify_attention`/`notify_result` と日次上限通知の設定では古いpayloadを破棄する。閉じる時は現在のpayloadだけを保持し、再表示時に三つのUI状態をfalseへ戻す。同じ通知内の比較再展開では `raw_mode` を保持する。文字列の等価性を通知IDに使わず、新規セット操作自体を境界とする。通常通知のコピー・fatal/font・Codexパス・日次上限の優先表示は既存経路を保つ。構造化payloadが有効なのは対応する現在の `message` が当該教材失敗で、fatalが優先していない間だけである。fatal解除後に古い教材診断が復活しないよう、fatalが発生した時点または表示切替時に古いpayloadを無効化する。`notification_text()` の一般文字列を日本語parseしてpayloadに戻さない。status行の要約は改行を除いた短い1行とし、hover/詳細に全文への入口を残す。

## 3. 表示・コピー・egui配置

初期画面は段階別タイトル、非登録と既存教材不変、`EvidenceCause` ごとの短い理由、次の操作、比較を開く操作を順に置く。原因の表現は原文・話者・対象を推測せず、`cause_detail` を確認できる技術情報に残す。比較を展開するとAI引用と生成要求時の固定元発言を上下に置き、取得不能なら理由を表示してその側の原文コピーを無効化する。比較上部に「読みやすい表示は原文の厳密一致を示さない」と示す。二本文の切替は一つの `raw_mode`。技術情報は別の開閉操作である。

限定rendererは `src/app/notifications/markdown.rs` に通知専用の純粋なtokenize/render関数として置く。入力は借用した原文 `&str`、描画対象は本文だけであり、リンク、画像、HTML、外部取得や実行を行わない。行頭 `#` 1–6、空行段落、実改行、行頭 `- ` / `* ` / `+ ` / 数字と `. `、対の `**` / `*`、単一/三連バッククォートを支援する。コード範囲は装飾を解釈しない。入れ子・不整合・未対応記号は文字として残す。rendererが返す装飾後Galleyを原文やコピーの所有者にしない。Raw表示も原文 `String` から作り、表示不能制御文字だけ可視表現にする場合は表示との差を示す。literal `\\n` を改行へ変換しない。改行・CRLF・tab・前後空白・Unicodeはコピー元で変更しない。

構造化通知の `diagnostic_copy_text(&MaterialDiagnostic) -> String` は開閉/Raw状態を引数に取らず、結果・理由・次の操作・取得済み技術値・双方の先頭240 Unicode scalar抜粋と取得不能理由を固定順で作る。241文字目がある側だけ省略を付け、各欄を「抜粋」と明記する。一般通知は既存「詳細をコピー」を維持する。各「原文をコピー」は `QuoteEvidence.quote` / `original` のcloneを直接 `ctx.copy_text` へ渡す。コピー後の成功メッセージで元通知を上書きしない。選択可能な本文は維持する。

`notification_window` の既存 `bottom_up` 固定footerと本文 `ScrollArea` を利用する。狭幅ではfooter操作を縦並びまたは折返し、本文・比較の各見出しとコピーを同じscroll領域で到達可能にする。ウィンドウをviewport内に収め、最大高さに応じて本文だけを縮める。標準1150×950/80%と狭幅820×650/160%で、初期要約、比較展開、Raw、技術情報、末尾、コピー、閉じるを検証する。文字ボタンは既存 `UiControls::ww_button` / `controls::Button` を使用し、日本語glyphの水平・垂直中心、focus、disabled、キーボード操作を維持する。選択中モード・開閉は文字で示す。読取専用白面、意味色、配色previewの境界を維持する。

## 4. 受入ID・所有・独立テスト入口

| ID | 主なmodule/状態・独立テスト入口 |
| --- | --- |
| N01 | `material.rs` stage/cause → `app.rs` 教材結果 → `notifications.rs` 初期要約。生成・保存案を別fixtureで起こし、deck/案/復習状態の前後比較。 |
| N02 | `QuoteEvidence` は失敗引用と固定snapshotからだけ構築。7番目理由・4組目回答・欠落/不正role・現在チャット変更を合成し、比較本文と対象を照合。 |
| N03 | `notifications/markdown.rs` の限定構文、原文切替、未閉鎖/未対応文字。両側で日本語/絵文字・実改行とliteral `\\n` を使い、描画とRaw/clipboardを別判定。 |
| N04 | `MaterialDiagnostic` が引用と固定元発言全文を保持。240/241、2000/2001、長文末尾markerの表示到達と原文コピー一致。 |
| N05 | `MaterialNotice` の初期false/同一通知内Raw保持、`diagnostic_copy_text` の状態非依存、原文コピーの完全一致。取得不能側のcopy不可。 |
| N06 | `notify_*` のpayload破棄と新規通知初期化。構造化→一般、同文再通知、閉じる/再表示、fatal/font、パス/日次上限、Storage読込互換。 |
| N07 | 既存validator一本化、`ready` と保存順序。`**`を跨ぐ/跨がないcontains、空白/長さ/件数/path順、表示操作後のAI呼出数・deck/記録・ディスク不変。 |
| N08 | `notifications.rs` の固定footer/scrollとrendererのローカル文字表示。隔離Windowsで2サイズ・zoom・末尾・日本語・keyboard・コピーを操作確認。 |

U5a本体: `src/material.rs`, `src/ai.rs`, `src/app.rs`, `src/app/notifications.rs`。U5b本体: `src/material.rs`, `src/app.rs`, `src/app/notifications.rs`。U5c本体: `src/app/notifications.rs`, 新規 `src/app/notifications/markdown.rs`。test authorは計画記載の同居 `cfg(test)` / harness / visual fixtureだけを別工程で所有する。新rendererファイルが不要な規模なら `notifications.rs` 内の私有moduleでもよいが、外部依存と全体Markdownは増やさない。共有ファイルはU5a→U5b→U5cの順に編集する。

## 5. リスクと引継ぎ

最も高いリスクは診断取得のための再検査が別の失敗へ変わること、旧案の登録境界、通知stateの残留である。保存案の詳細クリックでは既存 `ready` の当フレーム失敗を再確認し、`validate_evidence_detailed` が一致する根拠失敗のときだけ構造化する。通知本文のコピーと表示が検査結果を書き戻す経路は設けない。`String` wrapperとtyped版の両方で同一失敗・同一旧文言を照合する独立テストが必要である。

`Source::validate` が先に失敗した旧保存案では比較元を取得できない場合がある。N02の「取得不能」を表示し、存在しない引用を創作しない。Markdownは完全準拠ではないため、支援する構文とfallbackをWindows実描画で確認する。ネイティブIME・実AI・実データを使う受入は未実施として残す。実装前に設計レビューとplannerの所有確定、独立テスト期待値固定が必要である。
