# I2d-M 最終Post

2026-10-12。独立reviewer `media_export_post`（指定 `gpt-6-astra/high`）の静的確認とrunner証拠照合を親が保存した。**Post PASS、阻止指摘0、M-AC1–3を満たし、I2d-M限定単位をClosedとする。** 確定欠陥・要回答質問・任意提案も0件である。

対象はHEAD `81aaf978be3cac23b27acefcab85392aa375df39`、本体差分 `24fa10c..81aaf97`、作者差分 `a64ede0..24fa10c`。[設計/Pre](media-export-review.md)、[独立作者](media-export-tests.md)、[実装](media-export-implementation.md)、[独立runner](media-export-runner.md)を入力とした。reviewerは編集・test実行・再委任・公開をしていない。thread `01a12e03-feab-79e3-b89f-d7760236d12c`。runtime返却model/effortは未取得であり、指定値と区別する。

## 確認した証拠

- store SHA256 `12023DF049CFD57C034CD95B87FDF8D20A3368A8A2F24B733BA01457A99338D9`、chat_media `24171BE283A1D856A2752FA9CEE60D3AE4D43EE4386EFAC6C1B1C5C09C47D5F3`、固定Pre設計、独立oracle2区間の一致を確認した。既存oracleの変更はない。
- storeのguardは途中drop/再取得なしでcompletion Result評価完了まで連続保持する。全名の形式/ordinal衝突preflight、directory返却resolved pathの使用、member順、completion最後、非Windows拒否、write-block不解除を確認した。
- 3consumerは借用/生成後の`?`成功後だけ対象をclearする。途中Errでも元payload・他媒体・Progressを保持する。pickerのNone経路はexport/通知を呼ばない。Some(Storage)優先、現行起動のStorage::open、None時resolverの適用限界を照合した。
- 新25件の手製bytes/ink/全画素/CryptoAPI既知vector oracle、途中directory注入による境界拒否、既存5harness・H04/H07による実I/O/cleanupの証拠分担を確認した。binaryの途中Err保持はlibrary動的証拠とconsumerの`?`接合の静的確認を合成し、同一障害をbinaryで動的観測したとは扱わない。
- runner報告と実ログを照合し、fresh buildの4EXEの固定hash、新9＋16実名集合、全117名の非重複を確認した。有効行動は**114 PASS / 0 FAIL / fixture skip 3**、11run exit0、timeout0、子process/TEMP残存0である。

## 未検証と限定判定

symlink2件（権限不足）・別drive alias1件（fixture条件不足）は今回の成功へ数えない。既存identity境界は無変更であり、新接合のhardlink/junctionと関連境界回帰が成功しているため、本限定単位の阻止条件ではない。過去の管理者成功ログを今回実行の代用にはしていない。

全体test・release buildはI10/製品deliveryへ残す。native picker/IME/pen/mic/TTS/実API認証、敵対的TOCTOU、上記3条件、任意の失敗カスタムrootのNoneからの復元は今回未検証である。製品EXE未更新、音響履歴機能全体や製品全体の受入完了ではない。D2全体gateや早期試用への依存は別に判定する。
