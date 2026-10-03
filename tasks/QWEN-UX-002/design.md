# QWEN-UX-002 内部設計

2026-10-03。担当はdesigner、所有は本書のみ。計画者指定/起動指定は `gpt-6-astra / high`、実行metadata・開始/終了token値は未取得である。親承認済みのP-U01（AI接続ページ内でQwenを編集し、明示的なdraft接続確認を追加）と通信契約を入力とし、到着した [spec.md](spec.md) のQU-AC-001～018を照合した。仕様・設計の独立reviewと新画面の利用者受入は未実施であり、本書で合格へ変更しない。

## 根拠と変更境界

- `AGENTS.md`、`docs/process/development-team.md`、KPI規約、`api-and-interface-design`、`wordweave-egui-ui`を読了した。契約を型と境界に置き、UIは既存の共通controlsを利用する。新しいRESTサーバーは不要である。
- `QwenConnectionEditor::save` は `ApiHost::parse_for_region` → `Connection::with_host`/新キー → `CredentialStore::save` → baseline更新の順である。同じ正規化hostだけ保存済みキーを保持できる。この規則を接続確認にも適用する。
- `provider.rs` のTransportはPOST専用、SafeErrorは厳格な3フィールドwire形式である。GET確認を既存Transportへ混ぜず、診断の追加は承認された固定ErrorCodeとその文言で表す。
- `app.rs::update_ui` はQwen設定をconfirmingに含め中央領域全体を無効化する。inline化ではこのままでは入力不能となる。navigation/zoom抑止と中央領域の抑止を分ける必要がある。
- `qwen_reading_ui.rs::render` は既存controllerからviewを取得してactionを収集する。`dispatch_frame` の取消し/閉じる優先とcontrollerの送信準備IDを維持し、視覚構造だけ変更する。
- 原応答は残っていないため、今回のResponseInvalidの発生段階・原因は未確定である。JSON Object追加は予防策であり、実障害解消の証拠とはしない。公式調査は [evidence.md](evidence.md) を根拠とする。

## 受入条件と所有

| 仕様AC（計画AC） | モジュール・型・境界 | 独立テストの焦点 |
| --- | --- | --- |
| QU-AC-001～003（UX-AC-01/02） | `provider.rs::evaluate` の要求JSON、既存 `validate_feedback`、UIの固定error説明 | JSON Object指定、正常/mismatch/unassessable、不正・矛盾・途中終了の拒否、原文/秘密の非露出 |
| QU-AC-004～008（UX-AC-03/04） | `qwen_reading_ui.rs` の表示helper、既存ReadingView/QwenAction | phase別主操作、必須送信確認、取消し優先、旧結果排除 |
| QU-AC-009～011（UX-AC-05） | `qwen_settings.rs` のinline editor、`settings_ui.rs` の限定enabled境界 | Codex一般draft保持、Qwen専用save/cancel、失敗時保持、focus |
| QU-AC-012～014（UX-AC-06/07） | 新 `tools/qwen-audio/src/probe.rs`、lib公開追加 | GET/same-origin/1回/無音声、HTTP/JSON分類、期限・上限・非漏洩 |
| QU-AC-015（UX-AC-08） | QwenConnectionEditorのrevision、ProbeJob、ProbeState | 連打抑止、編集/保存/close/取消し、遅延完了破棄 |
| QU-AC-017（UX-AC-09） | controls、scroll body、固定footer、settingsのfocus | 日本語mesh中心、長文、820×650/80–160%、keyboardと可視領域 |
| QU-AC-016/018（UX-AC-10と継承境界） | CredentialStore、controller、既存テスト/合成preview、runner成果物 | 資格情報分離、snapshot、旧形式読込、固定差分、単独GUI/MCP、nativeと実API受入の区別 |

実装は二つの独立所有へ束ねられる。共通側は計画U1/U2（provider、probe、errorの固定code、lib）、app側はU3/U4/U5（qwen_settings、settings_ui、appの接続節、qwen_reading_ui、必要なvisual_check節）である。モデル/工程/gateの変更は本書では行わず、正確なdispatchは計画者が確定する。Rust本体と同居するテストは所有者間で直列編集する。

## 共通ライブラリ契約

新規公開型の最小契約は次の通りである。名称の機械的な調整は許容するが、既存POSTの公開契約は維持する。

```rust
pub struct ProbeRequest { /* private endpoint + ApiKey */ }
impl ProbeRequest { pub fn endpoint(&self) -> &str; }
pub trait ProbeTransport: Send + Sync {
    fn send<'a>(&'a self, request: ProbeRequest) -> TransportFuture<'a>;
}
pub struct ReqwestProbeTransport { /* private reqwest::Client */ }
pub struct ProbeSuccess; // message() -> 固定文言
pub enum ProbeError {
    InvalidConnection, Authentication, PermissionDenied, Unsupported,
    RateLimited, Provider, Network, Timeout, ResponseTooLarge, ResponseInvalid,
    ModelNotFound, Cancelled,
}
pub async fn probe_connection(
    connection: Connection, transport: Arc<dyn ProbeTransport>,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<ProbeSuccess, ProbeError>;
```

ProbeRequestはライブラリ内部だけで生成しDebugをredactedにする。key/bodyアクセスは公開しない。ProbeSuccess/ProbeErrorは固定列挙、Debug/Eqと固定Display/Errorを持つ。probeの失敗に音声送信状態を流用せず、既存SafeError/IPC wireへ新フィールドを追加しない。

検証済み `ApiHost` のcanonical末尾 `/compatible-mode/v1` だけを置換し、同一originの `/api/v1/models?model=qwen3.8-omni-flash&page_size=100` を生成する。資格情報snapshotを移動しBearer headerにだけ使用する。GETはbodyなし、POST/音声/英文なし、最大1回、retryなし。HTTPS-only、no_proxy、redirect none、connect 10秒、全要求とbody受信を合わせて30秒、body上限256KiBとする。Content-Lengthだけを信用せずstream蓄積前に累積長を確認する。cancel済みならtransportを呼ばず、要求/受信中はselectで中断しfutureをdropする。

成功には2xx、JSON object、`success: true`、`output.models`配列、各対象要素の文字列 `model`、完全一致 `qwen3.8-omni-flash` が必要である。正しい空配列/完全一致なしはModelNotFound、欠落/型不正はResponseInvalid、`success: false`はProviderである。401はAuthentication、403はPermissionDenied、429はRateLimited、3xxおよび404/405/501はUnsupported、その他非2xxはProviderとして確認失敗を示す。未対応でもSingapore/Hong Kongの非Workspace hostへ迂回しない。応答body・request URL・keyをerrorへ埋め込まない。

`evaluate` の要求に `response_format: {"type":"json_object"}` を加え、JSONを要求する既存promptを維持する。json_schema対応は推定しない。SSE/終了理由/feedback整合/参照抜粋/キー反射の検査は維持し、不正応答を修復して成功扱いしない。

親が確定した診断契約は既存ErrorCodeへの固定値追加である。JSON構文失敗をResponseJsonInvalid、フィールド型・必須項目・意味整合失敗をResponseFieldsInvalidとし、SSE等は既存ResponseInvalidを保持する。ResponseInvalidの説明文は利用者向けに変更するが、既存codeと3フィールド形式は維持する。新enumもserialize/deserialize往復とstandalone/MCP経路を検証する。新旧クライアントが未知enumや旧messageを受理できるとは主張せず、同梱成果物を共通版で検証する。

## 設定エディタ・非同期・保存順序

QwenConnectionEditorはbaseline Connection、region/host/Zeroizing key draftを所有する。`resolve_connection(&self) -> Result<Connection, SafeError>` をsaveとprobe開始で共用する。新キー未入力時はbaseline.with_hostだけを使用する。probeは資格情報storeを変更せず、成功後も自動saveしない。

editorは `revision: u64`、`ProbeState { Untested, Running, Completed(Result<ProbeSuccess, ProbeError>) }`、任意ProbeJobを持つ。ProbeJobは開始revision、CancellationToken、JoinHandle/結果receiverを持ち、Connectionをworkerへ渡す。editor Debugにdraftやsnapshotを出さない。UIは短いpollとrepaintだけ行い、block_on/JoinHandle待機をしない。runtime終了はabort/cancel後の非blocking shutdownで行う。

| 入力・イベント | 遷移・データ所有 |
| --- | --- |
| 手動「接続を確認」 | 入力を検証しsnapshot生成、revisionを捕捉、Runningへ。連打は新規送信しない |
| region/host/key編集 | region変更時のhost/key消去を維持。revision更新、既存job cancel/abort、結果消去、Untestedへ |
| 明示「確認を中止」 | job cancel/abort、Cancelled表示。自動再試験なし |
| worker完了 | editorが存在し、Runningかつ開始revision一致時のみ採用。close/編集後の完了は捨てる |
| Qwen保存 | まずprobeを無効化し、resolve → store.save → baseline更新 → key clear → editor終了。失敗時draft/baseline保持 |
| 「Qwen編集をキャンセル」/Escape/アプリclose | probeを直ちに無効化。dirtyなら既存破棄確認、編集へ戻っても試験を自動再開しない |
| 破棄/保存成功 | editor Dropでjobを止め秘密を解放。summary更新、Qwen編集ボタンへ一度focus/scroll |

一般settingsのsave/cancelはQwen資格情報を保存/巻戻ししない。Qwen保存済み状態と確認結果は別表示とし、「編集中の接続を確認」「確認しても保存されない」を明記する。baselineはsummaryのためだけ保持し、編集中のhost変更後に旧キーでprobeしない。保存済み設定の閲覧は同カードの詳細折畳みで可能とする。

inline化ではQwen editorがあることをnavigation/zoom抑止へ残す。一方、CentralPanelを無効化する条件からはQwen editor単独を外し、settingsページ内のカテゴリ、Codex欄、一般設定save/cancel、他設定欄を個別に無効化する。Qwenカードのeditorだけ操作可能とし、他modalが開いていれば従来通り全体を抑止する。close_requestedは既存破棄確認順を維持し、Qwen専用modalの毎frame描画は除去する。

## 音読画面・日本語・復帰

既存modal内を「短い目的と取消し注記」「読む例文カード」「録音/選択と音声名・長さ」「確認/状態/結果」の縦順にする。技術仕様、応答model/effort/usageは詳細折畳みへ移す。参照英文・選択音声・送信確認・課金不確実性は折り畳まない。注記 `※送信後の取り消しは、遠隔処理の停止・課金取り消しを保証しない。` は原文のまま上部へ置く。

Inputは録音/選択→内容確認、Confirmingは可視snapshot→明示送信、Runningは状態と取消し、完了/失敗は再練習とする。録音/音声検査/評価の実行中のみ対応する中止操作を表示し、通常Inputの重複取消しは除く。既存action優先順と実行前snapshot invalidationは変更しない。固定footerに「閉じる」と録音・結果破棄説明を置く。狭幅はfooterを縦積みにしてbodyへ残り高さを割り当てる。

ResponseInvalidは「AIの回答を評価結果として読み取れなかった。発音が悪いという意味ではない」を主表示し、再練習から音声と例文を再確認できる操作を添える。原因は不明であり「別英文だから」と断定しない。認証/host/keyのエラーはAI接続の該当欄へ誘導する。NotSent/MayHaveBeenSentの既存説明を残し、遠隔終了/課金取消しを断定しない。raw応答を見せず固定error codeだけ詳細へ置く。

通常ボタン/選択は `src/app/controls.rs` のButton/UiControls、詳細はww_collapsingを使う。日本語のmesh_bounds中心を両軸で揃える既存実装を再利用し、色だけで状態を区別しない。cardは明示的vertical layout、widthは親available_width以下、長文wrap、段落の行間は維持する。本文14前後/見出し20前後を初期値とし、nativeの日本語・80/100/125/160%で判読と高さを確認する。最大幅よりも820×650でfooter・主操作へ到達できることを優先する。

## テスト作成・実装への引継ぎとリスク

- 共通probe fixtureは独立したfake ProbeTransportを追加し、URL/call count/status/stream chunks/保留/取消しを記録する。実資格情報・実HTTPは使用しない。POST既存fixtureはJSON Object要求の検査を追加するがfeedbackの期待値を緩和しない。
- editor試験はfake CredentialStoreの保存回数/保存内容、未保存draft試験、変更先+空keyで送信0回、同一host保持、失敗時旧設定、連打、取消しと同frame完了、保存/閉じる後の遅延成功排除を含める。
- inline化でmodal前提のテスト操作を改め、背景save禁止・editor入力可能・close/破棄・focus復帰を検証する。親が確認したfixture注意点として、zoom設定後に3 frame warmup→frame内editor open→close→focus確認とする。未初期化時の12500論理rectを正規のviewportとみなして期待値を緩和しない。
- 合成previewにResponseInvalid、probe成功/認証/未対応/未掲載/取消し、input/confirm/running/result/mismatch/unassessableを追加する。描画境界に加えnative標準/狭幅・拡大で実画面を確認する。IME/実音声/実APIの証明とは区別する。
- 既存controllerを変更する必要が判明した場合は設計外の追加作業として親へ返す。SafeErrorへ自由な診断フィールドを追加する案はwire互換と秘密露出リスクから採用しない。JSON Objectが実失敗を解消するか、Workspace endpointの地域別稼働、catalog成功後の音声推論権限は未検証である。
- 設計成果物だけの確認はAC対応・リンク・契約整合である。実装後の焦点試験、全体test、双方release、独立reviewは [plan.md](plan.md) のgateを維持する。設計担当はコード/テストを変更せず、再委任/公開を行わない。
