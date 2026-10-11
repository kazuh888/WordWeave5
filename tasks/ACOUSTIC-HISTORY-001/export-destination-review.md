# I2d-H 外部書出し先共通境界の独立レビュー

## 設計Pre（2026-10-11）

既定reviewer backup_consumer_pre（指定gpt-6-astra/high）は **PASS、受入阻止0** と判定した。対象は[設計§5追補](design-i2.md)、SHA-256 `50BE63D85B75AF7E7D1F1EABB306698722CAF89AF6E46821FCB34B04E6A06380`。T08/T08b、plannerの最大2file所有（store本体/新integration、専用cfgはstore内）、新依存なしとの対応を確認した。

- 同一Mutexの取得処理だけを共有する外部guardは、正本write-blockを解除せず、poison/再入拒否を維持する。
- GUID/128-bit identity取得不能時の拒否は既定契約内。GUIDを返せないnetwork shareを対応済みに扱わない。
- 予定root自身/配下/未作成祖先を両modeで拒否する。Fileによる予定directory成分の閉塞と、Directory配下媒体生成による侵入を防ぐ。
- tempの事前保護検査、公開atomic_writeの契約を維持するprivate抽出、解決済み絶対pathと同一guardによる後続接合が整合する。H01–H07に独立oracleと失敗時観測がある。

非阻止の検証留意：H06の共有制限fixtureは実失敗を確認して採用する。FILE_READ_ATTRIBUTES要求はshare-mode=0で拒否されるとは限らない。作者はmetadata/identity取得の実失敗を独立確認し、成立しないfixtureを拒否試験の成功に数えない。根拠は[CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)。GUID取得の制約は[GetFinalPathNameByHandleW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)と照合した。

FFI ABI、実FSのidentity、alias権限、H06/H07実行証拠は実装/独立試験gateに残る。d-B/M consumer接続、I2-D2閉鎖、I10は未完了。reviewerはread-onlyで編集/build/test/再委譲なし。実行metadata/usage counter未取得。親が本記録を保存し、独立test作者へ留意点とともに引き継いだ。
