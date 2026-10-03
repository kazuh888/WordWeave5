# QWEN-AUDIO-001 レビュー記録

## U0 仕様・設計

2026-10-02、独立担当 `/root/qwen_review`（割当 gpt-6-astra / high）が修正確認後に合格と判定した。対象はspec.mdとdesign.md、最終レビュー時のdesign SHA-256接頭辞はD4656D44である。その後の親編集は承認状態の記録更新である。

初回の重要指摘と解消内容:

1. MCP親の取消しとGUI子の完了の競合時に、異なる終端状態を表示する余地があった。親だけが終端を確定し、子はCommitted応答を受けてから確定状態を表示する設計へ修正した。
2. 純粋描画テストだけでは、本番の設定保存・明示送信・取消しの経路を独立検証できなかった。本番GUIが利用するGuiControllerを偽資格情報store/transportで検証できる公開境界として追加した。

親は仕様のP01～P05、Tokyo URL入力表現、使用量の未取得扱い、max_tokens=4096（総課金上限ではない）、QA-AC-023の画面寿命を採用した。実装担当はCargo/src、独立テスト担当はtestsのみを所有する。両担当のモデルは計画どおりgpt-6.1-sol / highである。

## 実装レビュー

2026-10-02 最終判定PASS。独立担当 `/root/qwen_review`（割当 gpt-6-astra / high）が完成差分と[verification.md](verification.md)、native証拠を照合した。受入を阻止する残件はない。

1. P2: tools/callの標準params._meta.progressTokenを拒否していた。protocol metadataとtool argumentsを分離して検証し、文字列/整数token付きの3操作と不正metadata拒否の回帰を追加した。
2. P2: 不正資格情報blobを拒否した後にも未検証pointerへwrite_bytesしていた。有効な非NULL・1～2560 bytesのみcopy/zeroizeし、他はdereference/writeしないhelperへ修正した。合成blob4試験で確認した。

追加のRunning 180秒watchdog、AwaitingUserの無期限維持、stdout保持中の子終了検知、終端ACK、native子回収も照合した。親の出力サイズ懸念は、MCP入力128KiB/出力512KiBという実装上の区別により非該当と判断し、設計の表現だけを明確化した。

独立81 tests、fmt、locked release、STDIO smokeが成功。runnerの差分指紋 `3931AA62B9675C3F9B04B5B24B2C3CE576754E4A297EB5CB342C6AA9A8EDB473` とレビュー対象を照合した。EXE SHA-256は `9B36D56936D39852F0A24C66B153D5BBA475514F8FF65C5DF17F4041DEBB64CC`。

PASSは合意したオフライン開発の範囲である。実API接続、実資格情報保存復元、発音評価の妥当性、IME等の全操作、実Codex登録を合格と扱わない。
