# I1b-W1 独立runner結果

2026-10-07 / ACOUSTIC-HISTORY-001。W1の全37件は **37 passed / 0 failed / 0 ignored / 0 measured / 0 filtered out、exit 0、3.80秒** である。新規junction 2件を含む。独立Post-gateへ返す証拠であり、Post PASSや製品納品完了の判定ではない。

## 割当・入力・境界

計画者の `implementation.md` I1 runner指定は `gpt-6.1-sol` / `medium`、親の明示起動指定も同値である。実行model/effort metadataと使用量開始/終了counterは未取得であり、自己申告による確認や推計は行わない。子IDは `/root/m2_w1_runner`。測定取得範囲はW1限定、開始hash採取22:20:47 JST、終了hash採取22:21:17 JST（最終応答前まで）である。

AGENTS.md、development-team.md、KPI、ww-test-runner定義、wordweave-change全文を読了した。skillは保存境界の独立検証と結果報告だけへ適用した。入力は `history-w1-packet.md`、`m2-w1-gates.md` の独立Pre PASS、`history-w1-tests.md` と親が指定した生成EXEである。受入はI1b-W1 / W06 / AH-AC04/14、recovery junctionのprepare/publish拒否と全bytes保持を含む既存35件＋新2件である。

所有は本書と `artifacts/w1-runner-*` ログだけである。本体・テスト・期待値・manifest・設定を変更していない。Git操作禁止の引継ぎに従いGit照会・変更は行わず、以下の実測SHAで今回対象を固定した。基準HEADは計画書記載 `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d` であり、今回Gitから再取得した値ではない。旧snapshotの58,832 bytesは現試験70,553 bytesのprefixと完全一致した。旧SHAは `5D097873DF335BA6D152CF7522879D5870A43C5E44E5B443A748A9545C89098A` である。

## 対象固定

次のSHA256は実行開始前と終了後に一致した。test/EXEはauthor引継ぎ値とも一致し、再compileを省略した。これは同一EXEの独立実行であり、全依存ソースからの再build確認ではない。

| 対象 | 開始・終了SHA256 |
| --- | --- |
| tests/audio_history_store.rs | 7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4 |
| src/audio_history.rs | 77158A276489D1C859A59EE026720F5F314EFE8B38D23908AC0911689831AD81 |
| src/audio_history/store.rs | 4454F6391EC318C6E47B5629C3D8A692E4EFEDF6C604FBA4E7975D7D8E049603 |
| src/audio_history/recovery.rs | 2D804117C21E2427CEA44FD796B173667E6BE59CB5179E3E283B07347A7FF55B |
| src/lib.rs | 9D1E0ACB15F036EE9493CB5EB72A2318824760AC79BE5E30F12CD6835BED51AE |
| Cargo.toml | 1467C8C33A37730EF102521A3AD062FF7BB2267C4BBBE295B409BC283A5F0985 |
| Cargo.lock | 3B904B94D526EE414A11D7D446F086B52F6497BEB8035A155F2F311F3612239F |
| target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe | F1A00AE86F10C0045FD59CFEDE8FECC3BC9B18182F651A6E8A76321A1BCAA2F2 |

## 今回の実行

通常sandboxの既知junction error 5 / prepare Ioは反復しなかった。親の指示どおり合成fixture限定で `require_escalated` を使い、非管理者sandbox外で固定EXEを実行した。UAC、OS設定変更、実API、実データ、ユーザーアプリ停止は行っていない。

```powershell
& './target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe' 2>&1 | Tee-Object -FilePath 'tasks/ACOUSTIC-HISTORY-001/artifacts/w1-runner-full.log'
$w1Exit = $LASTEXITCODE
"runner_exit_code=$w1Exit" | Tee-Object -FilePath 'tasks/ACOUSTIC-HISTORY-001/artifacts/w1-runner-full.log' -Append
exit $w1Exit
```

実行結果はexit 0、37/37 PASS。各case名と結果は新規ログ `artifacts/w1-runner-full.log` にある。ログSHA256は `1DA75EB3DC84BCC1DFE131C555C4AF0E4E117A28817365AC8E92597A1D0709F3`。開始/終了の `Get-FileHash -Algorithm SHA256`、旧snapshotとのbyte prefix比較もexit 0であり、一致を確認した。追加の試験反復は行っていない。

## 未実施と引継ぎ

本単位はW1のみである。reader既存37件・file-symlink特権1件・root全体test・release buildは今回未実施であり、今回の成功件数には含めない。Windowsアプリ納品時の `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` はI10の別gateとして残る。M2変更後の独立回帰は親のfollowup後に行う。

実マイク・pen・TTS・IME・実Codex認証・実Qwen送信・音声一般精度は今回未検証である。native Windows junction fixtureの成功をこれらの実機受入へ読み替えない。失敗は0件、本体修正・期待値変更は0件。W1独立reviewerへ37件成功と固定対象・新ログを返し、今回は停止する。
