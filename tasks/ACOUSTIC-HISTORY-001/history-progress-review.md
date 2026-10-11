# I2a-S 独立レビュー

2026-10-08 / ACOUSTIC-HISTORY-001。reviewer `history_gate_review`、指定ww-reviewer / gpt-6-astra / high。実行metadata/使用量counterは未取得である。親がread-only報告を保存する。

## 静的コード・oracle — 阻止0、Post未了

入力はPre固定の[限定設計](history-progress-design.md)、[独立試験](history-progress-tests.md)、`artifacts/history-progress-before`との差分である。

- store.rs:403/438/521/555：公開Deserializeとtyped codecの共通Wire→normalize。必須nullable・重複raw JSON・未知top拒否、既存field/default、元wire版保持。
- store.rs:586/618/891/914：root/祖先/leaf安全検査、bounded read、存在確認後openのNotFound→Io、headの直接型変換、pure read無書込、load単一gate→under_guard recover。
- store.rs:116/199：Mの共通codec接合。旧v1 exact退避と正常旧v2退避不要を区別。
- M改訂は承認済み対応表に整合。commit本体/G/旧test不変、store M/S試験節は作者固定hash一致。

| 対象（レビュー前後一致） | SHA-256 |
| --- | --- |
| src/store.rs | 4C1287B2CDB4DE4621A39E06ED67DE2A181A272B7FB67507F4708C648FD00EEE |
| src/commit.rs | C1E7D086D12BB872B8B756616030EE12EFF4E150065D72539B3BBAA68CE022AD |
| tests/audio_history_progress.rs | 9C4BA7EB25451A88FBA4B8CC6FAE66B51B2ACF2C3E740A0A357FE376D7E870B0 |
| tests/audio_history_migration.rs | F78901C7A0D68D4DFC7FF46494C78ECF1D18C1989ABE5194A7584E98C8EE0CB8 |

受入残はS15＋改訂M25＋G24＋旧17の独立81件GREENである。RED未到達のtyped正例、拒否表後段、strict path全配置、write-block中read/head、移行Before/After境界も必要である。open後NotFound→Ioはコードで直接確認した。以後source変更時は差分/影響試験の再照合が必要である。編集・実行・再委任なし。N/R/I10/native受入は本判定の対象外である。

## 初回80PASS/1FAIL — oracle経路差の限定訂正承認

同reviewerが`artifacts/history-progress-implementation-progress.log`と現物を独立確認した。唯一の失敗は新integrationのs01で、直接to_value(f32)とJSON bytes経由Valueのfont_scale表現が0.800000011920929/0.8と異なる。f32型・既定0.8は変更前後同一であり、このログは値破損を示さない。本体hashは静的レビュー時と一致した。

独立作者が比較を`serde_json::to_vec(&read)==raw`の全field wire exact roundtripと、`read.settings.font_scale.to_bits()==p.settings.font_scale.to_bits()`の型値保持へ分離することは正当なoracle訂正と判定した。version2/validate/必須null/runtime1拒否assertは保持し、丸め・許容誤差・field除外は行わない。初回失敗は保存し、訂正後の当該target再実行・新test/EXE固定まで未解消、S Postも未了である。

## 最終Post — PASS / Closed

同reviewerが[独立runner](history-progress-runner.md)の新ログ・現物hashを照合し、S限定Post PASS、阻止0と判定した。独立81件成功、失敗0・ignored0。typed/head/path・移行fault・旧pending3状態等のRED未到達部分もGREENである。訂正したroundtripは全field exact wire＋f32 bitsと、nullable/default/runtime1拒否を維持した。

source/test/EXE等10対象のrunner前後と現物hashがすべて一致し、TEMP前後0、fmt/diff成功を確認した。初回80PASS/1FAILの履歴は保持する。open後NotFound→Ioは静的確認であり実レース再現ではない。編集・実行・再委任なし。N/R/I10/native・実データ受入は未達であり、SだけをClosedとする。
