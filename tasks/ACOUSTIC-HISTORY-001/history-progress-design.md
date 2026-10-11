# I2a-S: Progress codec / read / head限定接合

2026-10-08 / ACOUSTIC-HISTORY-001。担当は計画者指定 `ww-designer / gpt-6-astra / high`。実行metadataと使用量の開始・終了カウンターは未取得である。今回の所有は本書とdesign-i2.mdのS関連§2/3/7への統合だけであり、本体・試験・buildは実施していない。

入力は承認済み [spec.md](spec.md)、[design-i2.md](design-i2.md) §2/3/7（今回統合前SHA-256 `1375E2A126DBA2AEAD188903D3BA3E0575293B3C919C7721D5786EB771090EAF`）、[implementation.md](implementation.md) I2a-SとS限定計画delta、[M限定設計](history-migration-design.md) である。M限定Post PASSは [独立記録](history-migration-review.md) に保存済みである。本書を正本へ統合しても、独立Preと親による計画delta確定前にS本体へ着手しない。計画・担当モデル・品質gateを本書で変更しない。

## 1. 最小codec境界

`src/store.rs`内のprivate型・関数として次を固定する。`RootRef`は既存履歴DTOであり、新しいhead型や永続fieldを作らない。

```rust
#[derive(Debug)]
enum HistoryField {
    Missing,
    Present(Option<RootRef>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressWire {
    version: u32,
    settings: Settings,
    memories: BTreeMap<String, Memory>,
    reviews: Vec<Review>,
    suspended: BTreeSet<String>,
    #[serde(default)]
    deleted_entries: BTreeSet<String>,
    #[serde(default)]
    chat_action: Option<(String, crate::chat_action::Action)>,
    ai_calls: BTreeMap<String, u32>,
    study_seconds: BTreeMap<String, u32>,
    #[serde(default)]
    deck_versions: BTreeMap<String, String>,
    #[serde(default)]
    writing_logs: Vec<WritingLog>,
    #[serde(default)]
    japanese_drafts: BTreeMap<String, String>,
    #[serde(default)]
    chats: Vec<crate::chat::Conversation>,
    #[serde(default)]
    material_draft: Option<crate::material::Draft>,
    #[serde(default)]
    material_sources: Vec<crate::material::Source>,
    #[serde(default)]
    audio_history: HistoryField,
}

#[derive(Debug)]
struct DecodedProgress {
    wire_version: u32,
    progress: Progress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProgressReadError {
    InvalidDocument,
    UnsupportedVersion,
    LimitExceeded,
    UnsafePath,
    Io,
}

fn normalize_progress(wire: ProgressWire)
    -> Result<DecodedProgress, ProgressReadError>;
fn decode_progress(bytes: &[u8])
    -> Result<DecodedProgress, ProgressReadError>;
fn read_progress_typed(root: &Path, limit: u64)
    -> Result<Option<DecodedProgress>, ProgressReadError>;
```

署名はscaffold用の宣言であり、完成codecではない。`HistoryField::default()`はMissing、専用Deserializeは`Option<RootRef>`を読み、nullも必ずPresent(None)にする。通常の二重OptionやValue中間表現で欠落を推測しない。

公開`Progress`は既存field・メソッドを保ち、既定の`audio_history: Option<RootRef>`を追加する。Serialize deriveは維持し、Deserializeだけ手実装する。`Default.version`と構造validateはSで初めて2へ切り替える。Noneも`"audio_history": null`としてserializeし、省略しない。由来wire版を公開Progressへ追加せず、DecodedProgressが別に所有する。

公開DeserializeはProgressWireを直接deserializeしてnormalize_progressへ渡す。decode_progressはraw bytesから同じProgressWireを読んで同じnormalize_progressへ渡し、全入力を消費する。独立したv1/v2 fallbackや別の移行判定を作らない。これにより既存backup・pending・設定importの直接serde読取も同じ規則になる。

normalize_progressは元versionを保持したまま、v1のMissing/Present(None)をruntime v2/Noneへ、v2のPresent(root)をruntime v2/rootへ変換する。v1の非null、v2のMissingは拒否する。既存全fieldを移して構造validateし、rootの構造も検証するがclosure I/Oはしない。

topの未知field・重複field・欠落必須field・不正型はInvalidDocument、正常形の未知versionはUnsupportedVersionである。serde parse失敗と既存validate失敗をInvalidDocumentに対応付けるだけであり、表示文字列をparseしない。複数の不正が併存する入力のエラー優先順位は正本に未規定であるため、新しい優先順位を試験で要求しない。公開Deserializeのserde errorへの表示変換からtyped errorを復元しない。

既存defaultは上記fieldだけに維持する。Settingsのcontainer defaultによる実際の既定値を維持し、nested DTOの未知field規則を一括変更しない。ProgressWire全体をdefaultにして必須settings/memories等の欠落を許容してはならない。

## 2. pure readとhead

read_progress_typedはrootを絶対path化し、祖先からrootまでsymlink_metadataで通常directoryと確認する。欠損・取得不能はIo、非directory・symlink・Windows reparseはUnsafePathである。次にprogress.json leafを検査し、検証済み通常root配下のleaf NotFoundだけOk(None)とする。leafの非通常file・reparseはUnsafePath、その他取得失敗はIoである。

存在確認済みleafをopenして上限付きreadを行う。open/read時のNotFoundもIoであり、Noneへ落とさない。実際の読取量も制限し、LimitExceededを返す。decode_progressへ渡すまでにmkdir、backup、recover、intent変更をしない。検査はread/head自身で行い、callerのpending検査を理由に省略しない。完全な外部TOCTOU防止を追加するものではなく、Gと同じpath検査の脅威境界である。

既定公開`Storage::read_progress(&self) -> Result<Progress, String>`はtyped helperを使用し、Ok(None)だけDefaultへ変換、最後にerrorを表示用Stringへ変換する。既定公開`ProgressHistoryHead`の`CommittedHistoryHead::read_current`も同じtyped helperを使用し、分類を直接HistoryErrorKindへ対応付ける。Stringを経由・再分類しない。両者はgateを取得しない。head使用前のstrict pending判定は既定callerの責任であり、head自体はpendingを復旧しない。

Storage::loadはGのgate取得→recover_under_guard→read_progressの順へ接続する。閲覧成功条件へ履歴closureの健全性を追加しない。同期readのみであり、worker・async・取消し窓口を新設しない。write-block中もpure readは利用できるが、復旧付きloadはGの書込停止規則を維持する。

## 3. M / R接合と所有

Mのtarget2旧source検査はdecode_progressへ接続し、DecodedProgress.wire_versionで旧v1/v2を区別する。旧v1は元bytesをcontenthash退避し、旧v2は同codecによるstrict検証後に退避不要とする。runtime version=2だけで旧source版を判定せず、旧bytesを再serializeしない。decodeの固定分類からMigrationErrorへ直接対応付ける。target1のgate後no-op・通常saveの移行用旧read省略、新教材saveの既存旧metadata検査は変更しない。

M既定hookへ渡すtarget版は保存予定のwire版である。Sでは実際のv2通常save・新教材saveについてS-Mの退避順序を検証する。旧pendingのwire1 exact replayをruntimeのserializeで置換してはならない。検証済intentを旧sourceとするrecover hook・全事前検査後の退避・current already newの扱いはR所有であり、R-Mまで全first-v2-write保護は未達である。

M現物の`progress_wire_version(bytes) -> Result<u32, MigrationError>`はcommit本体からも呼ばれるため、署名を保ちstore内で共通decode結果のwire_versionを返す接続へ切り替える。旧source検査も同じcodecへ接続し、Mの暫定Version-only parserを第二のProgress読取規則として残さない。commit本体の編集は不要である。target1のprivate helperは従来どおりsource decode前にreturnし、target2では長さ検査後にdecodeする。

所有はplannerの最大4file案を反映する。`src/store.rs`本体とS専用cfg(test)、新`tests/audio_history_progress.rs`に加え、独立M作者が既存`tests/audio_history_migration.rs`とstore/commitの`i2_migration_tests`内の指定ケースだけを改訂する。`src/commit.rs`本体・旧test・G節は所有外である。従来の「最大3file」仮説はcommitのM専用試験を数えておらず、撤回する。共有storeはM作者→S作者→実装者の順で逐次移譲し、同時編集しない。

M oracleの変更前→変更後対応は [S限定計画delta](implementation.md) のケース名を正とし、次の意味に限定する。

| 指定Mケース群 | Sにおける改訂 / 維持する証拠 |
| --- | --- |
| `assert_kept` / load・daily backup | decode後runtime2・全sentinel保持、loadのbytes不変。初回保存wire2/必須nullとhash退避・日別退避の集合/各exact bytesを別々に照合する |
| 新教材save integration / commitのprivate入口、通常save private入口 | 実wire2保存へ改訂・改名し、既存公開saveと同じ本体を使う。faultなしはBeforeMigrationBackup→AfterMigrationBackup→次metadata操作の順。Before faultは全ledger不変、After faultは確認済みhash退避だけ残り旧metadata/pending不変。空seen・退避なし成功を期待しない |
| 手製wire1 pending replay | 3中断位置の元wire1 fixture、disk/pending archive exact、再recover不変、hash退避なしを維持し、runtime2 assertだけ別に置く。元文字列をruntime serializeへ置換しない |
| `m03_invalid_raw_version_or_structure_is_rejected_without_artifacts` | v2 root欠落はInvalidDocument、正しいv2/null・有効rootはstrict構造検証後に退避/fault/publisherなしで成功。v2不正root/未知top/重複は拒否、正常形の未知版はUnsupportedVersion |
| `m05_v1_shrink_does_not_add_migration_read_or_source_validation` | runtime2候補へ改訂・改名し、旧source上限超過と破損を別々に無変更拒否。private target1 no-opとtarget2上限拒否は維持する。M時点の公開v1縮小成功は固定した旧source/logの証拠であり、Sで公開v1候補を救済しない |
| 教材旧source/escaped intent上限・その他M01–04 | fault前停止、ledger不変、exact hash/競合/readback/path/I/Oのoracleを維持する。版dispatchに直接依存する指定fixture/名称以外は変更しない |

M作者は旧ケース→承認済み要求→新ケースの対応と追加/改名件数を返す。assert削除・ignore・両版許容による救済を行わず、旧fixture/hash/RED/GREEN記録を上書きしない。

## 4. RED前scaffoldと接続順

未接続の型/入口の追加だけなら既存M/Gの振る舞いを維持できる。独立作者が期待値と呼出署名を先に固定し、Pre後に実装役がstore.rs本体だけへ次のscaffoldを置く。関数宣言だけではRustはcompileできないため、新規関数には固定Errの非副作用bodyを置く。既存入口をstubへ差し替えない。

1. 公開Progressへ`#[serde(skip)] pub audio_history: Option<RootRef>`を追加し、既存Defaultのversion=1を保って新fieldだけNoneで初期化する。既存Serialize/Deserialize deriveとvalidate(version1)を維持する。skipはserializeとdeserializeの両方に効かせ、既存v1出力へnullを混入させない。新fieldのSomeを保存する製品接続はこの段階で禁止する。既存の外部struct literalは`..Default::default()`を使用しており、この追加だけで他file所有を広げる必要はない。
2. §1のprivate型・署名を追加する。HistoryFieldのDefaultはMissing、専用Deserializeは固定のserde errorを返す未接続stubとし、normalize/decode/read_progress_typedは固定ProgressReadError::Ioを返す。ProgressWireのderiveを成立させるtrait口だけを用意し、既存Deserialize・M helper・progress_wire_version・loadへはまだ接続しない。
3. 公開ProgressHistoryHeadのtrait実装とStorage::read_progressを追加し、それぞれ固定HistoryErrorKind::Io / 表示用Errだけを返す未接続口とする。head/readはI/O・gate・復旧を呼ばない。他のsave/commitやFaultsの入口は変更しない。
4. まず改訂前のM全試験・G全試験・既存store/commit試験がscaffold後も通る証拠を取る。その後、M作者が§3の指定oracle改訂、S作者が新試験を逐次所有してcompile/行動REDを取る。新しい正例がstub/v1の旧動作により失敗することを確認し、compile不成立や拒否系だけの成功を行動REDとしない。
5. RED後の本実装で一時skipを除去し、HistoryFieldの実Deserialize、共通codec、Progressのmanual Deserialize、Default2/validate2、read/head/load、Mのcodec接合を同じS単位で完成させる。Serializeのaudio_historyは省略しない。公開Deserializeをstubへ先につなぐ中間段階を作らず、M hookより先にv2書込だけを有効化しない。

scaffold前後はG/M/旧testの各module本文・既存prefixを固定比較し、全件成功を別ログで示す。S oracle改訂後はG/旧testとcommit本体が不変であることを比較し、M指定ケースの改訂差分と新証拠を別に示す。「G/Mの全試験不変」をS完成後にも主張しない。既定のfixture/helperに収まらないcompile依存や所有外の変更が見つかった場合は、REDのために既存入口を壊さず作者・plannerへ戻す。

## 5. 独立oracleと未達

| 接合 / 受入対応 | 独立した証拠 |
| --- | --- |
| S01 / I2-T01、AC04/16 | 全既存fieldへ異なるsentinelを持つ手製v1を公開serdeとtyped codecで読み、runtime2・wire1・全値保持。省略legacy fieldとSettings既定値を個別比較し、必須field欠落は拒否する |
| S02 / I2-T01 | raw JSONでv2 root欠落/null/有効root、v1非null、version欠落/不正/未知、未知top、version重複、audio_history null重複を個別化する。Value生成後の試験だけで重複拒否を証明しない |
| S03 / I2-T02のpure read部分 | pendingを含む手製treeの全file bytes/nameを事前採取し、read/headで不変。root/祖先欠損・非directory・reparse・I/O不明をNoneにしない。通常root下のleaf欠損だけdefault/None。上限ちょうど/超過とtyped head分類を確認する |
| S04 / I2-T01、S-M | 元v1 raw bytesを正常通常save/新教材saveから初回v2へ保存し、contenthash退避exact一致・保存wire2・再読runtime2を検証する。旧v2 sourceも同codecで拒否/成功を判定し、移行のためにruntime版をsource版と誤認しない |
| S05 / I2-T01、M回帰 | Mのv1 source fixtureはwire1を保持したまま再読runtime期待だけ仕様に合わせる。旧pending replayのdisk exact wire1と、decode結果runtime2を別々にassertする |

open後消失はIoとする分岐をprivate試験・独立コードレビューで確認し、共有processの時刻依存削除レースを必須oracleにはしない。新しい汎用filesystem注入口は本書で追加しない。

SだけではT02の全writer root保持、R-Mのrecover初回v2保護、旧backup exportの明示wire1投影、回復閲覧UI、AC08/09の処理再試行・取消しを完了としない。後続単位の責任を前倒ししない。本書の完了境界は署名・接合・所有論点・oracleの固定と文書検査までである。
