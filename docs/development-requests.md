# 少ない往復で開発を依頼する

目的は品質を落とすことではなく、同じ説明・調査・検証を重ねないことである。親のAstra／ultraは変更していない。まず文脈と工程を整理し、effort変更の効果とは分けて評価する。

## 依頼の最小形

```text
目的：チャット送信中であることが分かる表示にする。
範囲：チャット画面だけ。接続方式と教材保存は変えない。
完了条件：送信中／成功／失敗を区別でき、既存操作が維持される。
```

不具合なら「操作手順・期待した結果・実際の結果」を、分かる範囲で添える。画面の問題は画像1枚でもよい。実装箇所やテスト方法を利用者が調べる必要はない。

形式が揃っていなくても作業する。AIは、対象・安全性・完成判定が変わる不足だけを質問する。単純な文言変更に設計質問を並べない。「続けて」は未完了作業が一つなら有効であり、複数ある場合だけ対象を確認する。

完了した機能の次は、同じプロジェクトで新規タスクを開始する。上の依頼と必要なら `tasks/current.md` の参照で足りる。過去の全会話・全設計書を貼り直さない。同じ不具合の対応中は、無理にタスクを分けない。

## ファイルと役割

| ファイル | 役割 |
| --- | --- |
| [AGENTS.md](../AGENTS.md) | 親・子に共通する保全制約、質問条件、調査・検証の範囲 |
| [wordweave-change/SKILL.md](../.agents/skills/wordweave-change/SKILL.md) | WordWeaveの機能変更だけに使う、対象リスク別の確認事項 |
| [ww-scout.toml](../.codex/agents/ww-scout.toml) | Terra／mediumの限定的な読み取り調査役 |
| [tasks/current.md](../tasks/current.md) | 再開時に読む短い現状・未完了事項 |

AGENTS.mdは名前を変えて親用・子用に分けるのではなく、同じリポジトリの共通指示として使う。役割差分は `.codex/agents/ww-scout.toml` に置く。Skillは `.agents/skills/<固有名>/SKILL.md` とディレクトリで区別する。Skill配下の `agents/openai.yaml` はSkillの表示情報であり、子エージェントの定義ではない。[公式のAGENTS.md仕様](https://learn.chatgpt.com/docs/agent-configuration/agents-md)、[Skills仕様](https://learn.chatgpt.com/docs/build-skills)。

子を増やすだけでは節約にならない。公式にも並列化は単独実行より多くのトークンを消費すると説明されている。今回は狭い調査だけを短い新規文脈で渡し、実装判断・統合は親が担当する。全履歴を渡した子同士で報告を回さない。[公式Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)。

## 反映と制約

- 次の新規タスクでSkill・役割の認識を確認する。認識されなければCodexを再起動して確認する。今回の構文確認は、将来のセッションで自動読込されることの実証ではない。
- 現在の会話の過去履歴や、ホストが既に渡したSkills一覧を、このAGENTS.mdで取り消すことはできない。
- **既存Skillsのプロジェクト限定無効化は未実施である。** 利用者から許可を得たが、調査したCodexソース `codex-rs/config/src/skills_config.rs::skill_config_rules_from_stack` はUser/SessionFlagsだけを受け付け、Project層を除外している。効果のない `.codex/config.toml` を作成して「無効化済み」としない。
- 個人設定、他プロジェクト、プラグインのインストール状態は変更していない。CLIの実行時設定は代替候補だが、Desktopの今の作業をそのまま切り替える方式ではないため、今回は起動ラッパーを追加していない。
- 将来プロジェクト限定の切替が確認できたら、候補は `agent-skills:using-agent-skills`、`context-engineering`、`planning-and-task-breakdown`、`incremental-implementation`、`code-review-and-quality` と、`compound-engineering:ce-work`、`ce-plan`、`ce-code-review`、`ce-simplify-code`、`lfg` の重複する工程指示である。セキュリティ・データ保全・専門技術の指示まで一律無効化しない。

公式はAstraに過剰・重複した指示を与えず、必要な検証が通った後の追加作業を理由なく続けないことを勧めている。今回の小さなHarnessはその方針による。[最新モデルガイド](https://developers.openai.com/api/docs/guides/latest-model)、[Skills・プロンプト再考](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra)。

effortを下げる比較を希望する場合、次の小さな変更でhigh等を利用者が選び、同種の作業で品質・手戻り・消費を比較する。現在のultraを無断で下げたり、一定の削減率を約束したりしない。[測定の基準](process/improvement/kpi.md)。

戻す場合は今回追加した指示・Skill・役割とCODEX_START.mdの今回の差分だけを戻す。アプリや学習データを初期化する必要はない。後から加えた他の変更をまとめて戻さない。
