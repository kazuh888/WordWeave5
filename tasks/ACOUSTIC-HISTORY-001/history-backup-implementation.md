# I2b-B 本体実装 / AH-AC16・I2-T06

2026-10-11。担当implementer、planner指定 `gpt-6-astra/high`。実行metadata・開始終了usage counterは未取得である。承認済みdesign-i2.mdのb-B追補、implementation.mdの最大3file delta、独立Pre PASSを入力とし、wordweave-changeを適用した。完了境界は本体2fileの接合と焦点自己検証であり、独立runner/Post・利用者受入とは区別する。

## 変更範囲

- `src/store.rs`：祖先metadata走査を `progress_root_exists` に共有し、RequireExistingを通常readerへ維持した。backup bridgeだけAllowAbsentを使い、存在確認後にpending拒否→従来typed read→runtime値へ進む。最初の不存在だけNone、先行file/reparse/I/O不明は拒否する。rootの再検査、leaf metadata・open・bounded read・strict decodeを維持した。
- `src/backup.rs`：memory退避は単一guard内でpending・最新disk root・closureを確認し、currentの全非履歴fieldを保持する。exportは同じguardを完了manifestまで保持し、candidate/diskのhistoryを宛先作成前に拒否する。型付け済み値のlegacy wire1投影と最終bytes上限を追加した。prepareは元bytes hashとstrict decodeの後に非null rootを拒否する。restoreは既定の日別ai_calls最大値と媒体処理を保持し、`save_progress`の実保存値を返す。

テスト・fixture・期待値、本体第3file、公開API、依存、設定は変更していない。最初のscaffoldは未接続の固定Errだけであり、作者の行動RED後に本体へ置換した。実データ・live AI・ネットワーク送信・commit/pushは使用していない。

## 自己検証

作者の42 unique（integration28＋store8＋backup6）を全てGREENへ到達させた。既存回帰33 uniqueもGREENである。合計75 unique、行動不合格0、ignored0、timeout0。ただし下記の複数EXE世代/環境に跨る証拠であり、最終sourceの独立一括検証ではない。

| 対象 | 通常sandbox | 限定sandbox外 | 合計 |
| --- | --- | --- | --- |
| audio_history_backup | 26 PASS | 媒体再検証・late拒否の2 PASS | 28 |
| store::i2_backup_path_tests | 7 PASS | junction 1 PASS | 8 |
| backup:: | 4 PASS | 旧媒体・assets 2 PASS | 6 |
| store::i2_progress_tests | 10 PASS、junction setup error5が1件 | 該当1 PASS | 11 |
| audio_history_progress | 4 PASS | なし | 4 |
| i2_history_main_consumer | 16 PASS | なし | 16 |
| backup_confirmation_ | 媒体setup error5で未到達1件 | 該当1 PASS | 1 |
| json_restore_preserves_the_latest_unsaved_chat_draft | 1 PASS | なし | 1 |

主なコマンドは `cargo test --locked --lib --test audio_history_backup --test audio_history_progress --bin wordweave5 --no-run`。最初の実行でsession IDを取得漏れし終了metadataを回収できなかったため、logged no-runを開始した。この実行は先行buildのlock待ち後に再compileし5分26秒、exit0で終了した。重複buildを効率上の手戻りとして残す。既存dead-code、incremental hard-link fallback、既存float literal warningがあり、compile errorは0である。

生成EXEはStart-Process Hidden・process限定TEMP/TMP・stdout/stderr別ログ・45秒上限で実行した。最初の42件と通常store10件は先行buildのEXEで実行し、直後のhashを採取する前にlogged buildで更新されたため、その世代のEXE hashは未取得である。後続33件のうちreader junction1/progress4/bin18件はlogged buildのEXEで成功した。途中のsource変更はrustfmt指定の本体4箇所の折返しのみで、挙動・テストは不変である。logged build開始後に整形したため、最終整形sourceとの再build照合は独立runnerへ残す。

実行の不成立を成功へ含めない。progress初回は `--test-threads` の重複指定で0件・exit101となり、引数訂正後4件成功した。reader junction最初の外実行はlink中EXE使用中によりStart-Processが失敗し0件、build終了後のretryで1件成功した。sandbox制限は合成媒体のAssetStore::putとjunction作成におけるerror5であり、元syscallを推測しない。外実行は同一EXE・exact caseに限定し、UAC/OS設定変更を行わなかった。

専用TEMPは `artifacts/i2b-b-impl-temp` と `artifacts/i2b-b-impl-external-temp`。失敗したUI fixtureの残存 `ww-ui-15488-1791718500877051200-0` は絶対pathが所有TEMP内と確認して削除した。合成データのみであり復旧不要、両TEMPの子要素は最終0である。

## 差分・oracle・固定hash

作者 `logs/history-backup-author-oracles.txt` のstore新moduleとbackup cfg(test)全文は対応範囲との文字列照合で一致した。integration hashも作者固定値と一致した。`rustfmt --edition 2021 --check src/backup.rs src/store.rs` と `git diff --check` は成功した。既存編集・親の文書更新は保持している。下表は追加B-WB1の作者編集前の固定値であり、追加後のstore/lib hashは作者・runnerの別証拠へ分ける。

| 対象 | SHA-256 |
| --- | --- |
| src/backup.rs | `12A20A9E5C509574B078A77C47E6B836A112A81EAAD0B8419CDC8E028DC84261` |
| src/store.rs | `91DF6C7CC92ABF8DEB2800C08B6B3C47D02F5F6C089ECCE89484FB05CDBB5118` |
| tests/audio_history_backup.rs | `B907A3F6BCAF195000669CC2B7F954FC4323AC7832B0CDE5A8DE63B3B76CEE0F` |
| logged build lib EXE wordweave5-b3388a7bbef44bb4.exe | `A6847D137A3A93D7266514B4D54A5FC65B35089A935ED532340A2C548EA3DCA9` |
| logged build bin EXE wordweave5-72e8adffa8f9ef2c.exe | `3E66F8F482D81191E9E7CC9B8B6AB5F69BDDF8419FF9209E8F754CB22A9A5E32` |
| logged build audio_history_backup-bb0141422dff8e3c.exe | `2F55A25BD540AF390E271884CA5D428055E1B4B89716EA2D95D5C6BF8EA89CF4` |
| logged build audio_history_progress-892b5d70a3c847c8.exe | `530CCF72C7550181A73A21C1D2056947276B323E6F1894CCBA5068FE693A8580` |

## 残る確認と引継ぎ

新backup入口のwrite-block拒否を直接確認する試験は42件に含まれず、親が限定Pre PASS後に追加B-WB1の1件を独立作者へ割り当てた。本体の推測fallbackは加えない。write-block下の外部export成功はI2d-H/Bの未実装接合であり、本単位の通常guardで実現済みとはしない。実path置換競合/metadata不明の実証、ネイティブ受入、I10全suite/release、独立runner/Post、利用者受入は未了である。

ログは `logs/history-backup-impl-*`。build、通常integration/store/backup/progress/main-consumer/confirmation/draft、外部exactケースにstdout/stderrを分離している。失敗/0件の初回ログも保持した。実装者はsourceを停止し、親へ所有権と上記未確認事項を返す。
