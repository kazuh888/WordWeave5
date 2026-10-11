# 文書検証記録

2026-10-04 / ACOUSTIC-HISTORY-001 / C2 / test runner。初回の固定入力に対する結果を保持し、末尾に統合後の最終検査を追記した。

対象は文書設計のみである。計画者指定は `gpt-6.1-sol` / `medium`、実行model/effort metadataと利用量の開始・終了カウンターは未取得である。AGENTS.md、開発体制、KPI、wordweave-change全文を読み、同Skillのリスク別検証・報告部分のみ適用した。所有は本報告のみであり、仕様・設計・試験計画・ソース・設定は編集していない。

入力handoffは親からのC2文書検査依頼と、plan.mdのtest runner割当、spec.mdのAH-AC01–18、design.mdの対応表、tests.mdの独立受入試験計画である。具体仕様と試験は承認候補であり、テストコードとfixtureは未作成である。検証結果は文書構造の合格であり、独立設計レビュー、利用者の採否、動作受入とは別である。

## 対象revisionと差分

HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。未コミット差分を検査した。検査時点はtracked Markdown 6件の18行追加と、untracked Markdown 7件である。以下のSHA256は実ファイルbytesに対する検査時の値であり、本報告作成前の入力を固定する。後からplan/results等が変わった場合、その新内容は本結果の対象ではない。本報告自身はこの13件に含まれない。

| パス | SHA256 |
| --- | --- |
| docs/reuse/README.md | 22728949D61B852ED7FEDCA918E4DE985F0F67AD5DC662F2DDFBAEF1EEF30AAC |
| docs/reuse/acoustic-analysis.md | C147589E745EE44E9F8C918EC406593BF508D64484C5123E7422E605E5A72D8D |
| docs/reuse/audio-io.md | 994B5B6549FF1428221A4F503DF2A701E49A2539CDD1A0117A17FC55F580448E |
| docs/reuse/asset-store.md | 6B4738E55C4FB3AEF2B794EDBC20EC9EF9EB4D5F0DA261B380D78888492785BD |
| docs/reuse/qwen-audio.md | 8E1F69009C5853743DE4135469E30BDD12EC37343BF1DA426BFF681D2E4D76C9 |
| docs/reuse/framework.md | BD3183C85D64837DA52F760F9A12F21DCCEEBF94C343CB606C93F397726EEFD3 |
| docs/reuse/library-candidates.md | AB46177869B9FD7D0755E414B599A4F78277DEFE7C1C284F32E9317E24169780 |
| tasks/ACOUSTIC-HISTORY-001/design.md | C2EE5F0EF9B24CB2ECA0D100D3D9802068F4C2E5A239D21CB4021866AD8E51DC |
| tasks/ACOUSTIC-HISTORY-001/evidence.md | C9F7C1657FBE60741F436D3050E9579EF960CB935BCEC5CB981A292C0B7A90CD |
| tasks/ACOUSTIC-HISTORY-001/plan.md | E47AF4D6D4C871392B4CBECD2527DEF005B7F8FC50EAF1E0745AF235C5ACD7E8 |
| tasks/ACOUSTIC-HISTORY-001/results.md | 9074ABD0DD5895F8859F60FDBCF1010A06CBC36C45C8121301704ED1EFA0F844 |
| tasks/ACOUSTIC-HISTORY-001/spec.md | C2DED29127FDF2B8A3746D909E6E7BA83B7C74DF9F3970BB8F206C6641055332 |
| tasks/ACOUSTIC-HISTORY-001/tests.md | BCB5F35DEC52CDF4802E7546908E55F5116BB659D65028DBD05AF15657105521 |

## 実施結果

| 検査 | exit code | pass / fail / skip | 結果 |
| --- | --- | --- | --- |
| git diff --check | 0 | 1 / 0 / 0 | tracked差分の空白エラーなし |
| 相対リンクの存在 | 0 | 149 / 0 / 7 | 13文書。外部URL7件は対象外、fragmentのみ0件。fenced codeは除外 |
| 行末空白 | 0 | 13 / 0 / 0 | untrackedを含め全行直接検査、違反0件 |
| AC対応表 | 0 | 54 / 0 / 0 | spec/design/tests各18件、AH-AC01–18、重複・欠落0件 |
| 変更種別 | 0 | 13 / 0 / 0 | 全changed/untrackedはMarkdown。code/Cargo/binary変更0件 |

pass数は文書・リンク・対応行の検査単位である。アプリのテスト成功件数ではない。合計を動作試験件数へ換算しない。リンク検査はファイル存在の確認であり、外部URL取得や見出しfragmentの解決を確認していない。AC対応検査はIDの一意性と集合一致であり、仕様意味の妥当性は独立レビューの担当である。

通常のshell起動は `helper_unknown_error: apply deny-read ACLs` によりprocessを作成できなかった。Node読取りもtrusted processが終了し、内容未取得であった。いずれも文書検査の不合格やexit codeとして扱わない。その後、同じ目的の読取り・検査を `require_escalated` で実行し成功した。自動承認の拒否はない。

## コマンドとログ

cwdは `F:\Kazuhiro\GitHub\WordWeave5`、PowerShellである。規約/入力の読み取りcommandはexit 0。`git status --short`、`git rev-parse HEAD` はexit 0。差分確認commandの `git diff --stat` は6 files changed, 18 insertions、最後の `git diff --check` はexit 0で出力なしであった。

以下は実際に実行した文書検査commandである。exit 0、stdoutの要約と固定hashは本報告に保存する。別の過去ログは使用していない。

```powershell
$scope = @('docs/reuse/README.md','docs/reuse/acoustic-analysis.md','docs/reuse/audio-io.md','docs/reuse/asset-store.md','docs/reuse/qwen-audio.md','docs/reuse/framework.md','docs/reuse/library-candidates.md') + @(Get-ChildItem tasks/ACOUSTIC-HISTORY-001 -Filter *.md -File | ForEach-Object { 'tasks/ACOUSTIC-HISTORY-001/' + $_.Name }); $bad = 0; $links = 0; $external = 0; $anchors = 0; $whitespace = 0; foreach ($p in $scope) { $lines = [IO.File]::ReadAllLines((Join-Path (Get-Location) $p)); $fence = $false; foreach ($line in $lines) { if ($line -match '\s+$') { $whitespace++; Write-Output "WHITESPACE $p" }; if ($line -match '^\s*(```|~~~)') { $fence = !$fence; continue }; if ($fence) { continue }; foreach ($m in [regex]::Matches($line,'\[[^\]]*\]\(([^)]+)\)')) { $target = $m.Groups[1].Value.Trim(); if ($target -match '^[a-zA-Z][a-zA-Z0-9+.-]*:|^//') { $external++; continue }; if ($target.StartsWith('#')) { $anchors++; continue }; $target = ($target -split '#')[0].Trim('<','>'); $links++; $resolved = Join-Path (Split-Path (Join-Path (Get-Location) $p)) $target; if (!(Test-Path -LiteralPath $resolved)) { $bad++; Write-Output "MISSING $p -> $target" } }; }; Write-Output ('HASH ' + $p + ' ' + (Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash) }; Write-Output "SUMMARY files=$($scope.Count) relative_links=$links missing=$bad external_skip=$external anchor_skip=$anchors trailing_whitespace=$whitespace"; foreach($p in @('spec.md','design.md','tests.md')) { $text = [IO.File]::ReadAllText((Join-Path (Get-Location) ('tasks/ACOUSTIC-HISTORY-001/'+$p))); $rows=@([regex]::Matches($text,'(?m)^\| AH-AC(\d{2})[^\r\n]*') | ForEach-Object { $_.Groups[1].Value }); $wanted=@(1..18 | ForEach-Object { '{0:D2}' -f $_ }); $missing=@($wanted | Where-Object { $_ -notin $rows }); $dupes=@($rows | Group-Object | Where-Object { $_.Count -gt 1 }); Write-Output "MAPPING $p rows=$($rows.Count) unique=$(@($rows|Select-Object -Unique).Count) missing=$($missing -join ',') duplicate=$($dupes.Name -join ',')"; if ($missing.Count -gt 0 -or $dupes.Count -gt 0 -or $rows.Count -ne 18) { $bad++ } }; $paths=@(git diff --name-only HEAD) + @(git ls-files --others --exclude-standard); $nonmd=@($paths | Where-Object { $_ -notmatch '\.md$' }); Write-Output "SCOPE changed_or_untracked=$($paths.Count) non_markdown=$($nonmd.Count)"; $nonmd | ForEach-Object { Write-Output "NONMD $_" }; if($bad -gt 0 -or $whitespace -gt 0 -or $nonmd.Count -gt 0) { exit 1 }
```

```text
SUMMARY files=13 relative_links=149 missing=0 external_skip=7 anchor_skip=0 trailing_whitespace=0
MAPPING spec.md rows=18 unique=18 missing= duplicate=
MAPPING design.md rows=18 unique=18 missing= duplicate=
MAPPING tests.md rows=18 unique=18 missing= duplicate=
SCOPE changed_or_untracked=13 non_markdown=0
```

## 未実施・次工程へのhandoff

今回のcargo test/buildは文書のみのためN/Aである。実装後のWindows提供では `cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5` と対象ACのfocused regressionを行う。保存・取消し・認証・広いUIの独立レビューは維持する。

原音保存/復旧/backup/trash、合成信号の数値oracle、解析資源量・取消し、二host適合、Qwen代替transportの試験はtests.mdにある将来ケースであり、今回は実行0件である。未実装のまま動作のpass/failを付けない。

Windowsネイティブの録音・再生・マイク・IME・支援技術・UI受入、実音声のF0精度、Qwen内容・文一致判定の品質、実認証、Praat導入/配布/license/性能は未検証である。pen/TTSも試験していない。実APIや実学習データは使用せず、ユーザーのアプリ停止やEXE置換をしていない。

親への引継ぎは、上記固定入力に対する文書検査の成功、未承認のAC/解析profile/誤差基準、独立レビュー判定、後続実装・実機gateである。入力が変わる場合は影響した文書検査だけ再実行する。

## 統合後の最終静的検査

2026-10-04。親から、仕様/設計/試験計画のレビュー指摘修正、planner P1、結果・レビュー・索引・利用量記録の統合完了を受領した。入力変更を理由に静的検査を再実行した。独立レビュー再確認で指摘解消済みとの親handoffを受領しているが、本担当の静的検査を独立設計レビューの代替にはしない。最終検査・報告保存後の時刻観測は `2026-10-04T00:38:54+09:00`（`Get-Date -Format 'yyyy-MM-ddTHH:mm:sszzz'`、exit 0）である。実行開始の正確な時計値は未取得である。同時に本報告117行の行末空白0件を確認した。

HEADは引き続き `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。検査対象は前回13文書とreview.md、tasks/current.md、docs/process/improvement/measurements.mdの計16文書である。変更/新規一覧は本報告を含め17件、すべてMarkdownである。validation.md自身は自己hash対象から除外した。

以下は前回から更新された入力と追加入力のSHA256である。前回表にあるREADME/acoustic-analysis/audio-io/asset-store/qwen-audio/framework/library-candidates/evidenceの8文書は、今回取得したhashも前回表と同一であった。これら8件と以下8件を合わせて今回の16入力を固定する。

| パス | 最終検査時SHA256 |
| --- | --- |
| tasks/current.md | 89D2964A0BB35B50D89B454F9C1F2F1663470184C6945EA300087932CCE439EB |
| docs/process/improvement/measurements.md | 05A01C7D0FE20E6BD64274828505521FCFCADAA5942CB5CD307376F7A869CEE0 |
| tasks/ACOUSTIC-HISTORY-001/design.md | 027E206DA853169BCCFE3D9F7007FC848ADD69D43E428CC79ED8758B4DC67210 |
| tasks/ACOUSTIC-HISTORY-001/plan.md | E7D9E183888A2BA142C2CAE07AE25E18BFBB5D27FF07A13B63160D039869F1F8 |
| tasks/ACOUSTIC-HISTORY-001/results.md | CB2D72DCDBD2D7EBA94AA9BFAB1A194B89F9A3A498B3A36CB8CC3F5199DC2CCD |
| tasks/ACOUSTIC-HISTORY-001/review.md | 362B6925A15234DCF77ECA29AFDC6424913F88FFEACFF6ECB28F884748173341 |
| tasks/ACOUSTIC-HISTORY-001/spec.md | 51FE915FC3A2A9E5F167C54734BDCF54D45A53282EED83BAF25CFA6C638C703E |
| tasks/ACOUSTIC-HISTORY-001/tests.md | 5CB06CEC12C1C84B6CFEA4EDC81B4444D7C6236FB5F9E6B44489BF5659D178B8 |

計画のC2-docチェックボックスは、親が本結果受領後にstatusのみ更新する予定である。その行は後続の軽微な統合記録であり、上記plan.mdのhashがその更新後も一致するとは主張しない。将来の内容改訂は新入力である。

最終commandは前掲検査commandのscope代入を次の第1行へ置換し、第2行をscope代入直後へ追加、末尾exit判定を第3行へ置換して実行した。他の検査bodyは前掲と同一である。この差分で実行commandを再構成できる。cwdとPowerShell、読取り専用のrequire_escalatedも同一であり、process exitは0である。

```powershell
$scope = @('docs/reuse/README.md','docs/reuse/acoustic-analysis.md','docs/reuse/audio-io.md','docs/reuse/asset-store.md','docs/reuse/qwen-audio.md','docs/reuse/framework.md','docs/reuse/library-candidates.md','tasks/current.md','docs/process/improvement/measurements.md') + @(Get-ChildItem tasks/ACOUSTIC-HISTORY-001 -Filter *.md -File | Where-Object { $_.Name -ne 'validation.md' } | ForEach-Object { 'tasks/ACOUSTIC-HISTORY-001/' + $_.Name });
git rev-parse HEAD; git status --short; git diff --check; $diffExit=$LASTEXITCODE; Write-Output "DIFF_CHECK exit=$diffExit";
if($diffExit -ne 0 -or $bad -gt 0 -or $whitespace -gt 0 -or $nonmd.Count -gt 0) { exit 1 }
```

```text
DIFF_CHECK exit=0
SUMMARY files=16 relative_links=206 missing=0 external_skip=17 anchor_skip=0 trailing_whitespace=0
MAPPING spec.md rows=18 unique=18 missing= duplicate=
MAPPING design.md rows=18 unique=18 missing= duplicate=
MAPPING tests.md rows=18 unique=18 missing= duplicate=
SCOPE changed_or_untracked=17 non_markdown=0
```

| 最終検査 | exit code | pass / fail / skip |
| --- | --- | --- |
| git diff --check | 0 | 1 / 0 / 0 |
| 相対リンク存在 | 0 | 206 / 0 / 17（外部URL） |
| 行末空白 | 0 | 16 / 0 / 0 |
| AC01–18対応 | 0 | 54 / 0 / 0 |
| 全変更種別 | 0 | 17 / 0 / 0 |

code/Cargo/binary変更0件である。初回同様、これらは文書検査の件数であり、動作試験0件、cargo/Windows提供gateは今回N/A、ネイティブ・精度・実API/認証の未検証事項は前節のままである。
