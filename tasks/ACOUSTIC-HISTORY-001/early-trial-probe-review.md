# ET1 probe 限定Pre

2026-10-13。独立reviewer `trial_dependency_gate`（gpt-6-astra/high指定）が、[限定probe設計](early-trial-probe-design.md)をread-onlyで確認した。対象は正本[外部契約](early-trial-contract.md)のprobe/失敗段階とET1の5file所有を具体化するdeltaだけであり、既存ET0やD2レビューを反復していない。

## 初回

設計SHA256 `CAB517D1385717F523EFB716A7590A657F73EFD7F765DAEB6439AB9C656294B0`はCHANGES_REQUIRED、阻止1件であった。

ET-P01/P2：82行の「登録解除の検査以外でOnceLockをresetしない」は試験でのreset例外を許しており、固定TrialRootsを解除しない契約と矛盾した。observerのRegistration解除と本体OnceLockが混同される余地があった。

他範囲には阻止なし。9到達点、実create衝突とwrite/sync/remove決定的注入の区別、File drop後のcleanup一回、衝突sentinel保持、remove成功後の本体登録、失敗stage/reason、通常ledger不変、cfg限定と5file所有を確認した。実disk-full/ACLエラーを再現した証拠とは扱わない。

## 是正後

designerは82行を「解除するのはthread-local observer Registrationだけ。本体TrialRoots OnceLockは試験でもresetせず、状態分離は有限timeout子processで行う」へ是正し、短い指摘追跡を追記した。独立reviewerは是正差分とhashを照合し、他の設計変更がないことも確認した。

確定SHA256 `40A12BE64F07136DEBE410FB52C22A124D5AC89CDB8D1666E5E19FCF52C05D81`は**限定再Pre PASS、ET-P01設計上Closed、残阻止0**である。親は作者へこのsnapshotを渡し、store専用probe cfg(test)を既定5file内に追加する所有を確定した。sourceの故障口や本体OnceLock resetを設ける許可ではない。

本体・試験実行・製品cfg除外の実証は後工程であり未検証。reviewerによる編集・試験実行・再委任は0。担当threadは依存確認と同じ `01a13514-2714-7ae0-81a8-0953439c52f0`、runtime返却値とusageは親の[今回観測](usage-observation-20261013.md)で別記する。
