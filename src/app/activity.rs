use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Activity {
    Study,
    Generate,
    Connection,
    Chat,
    Material,
    Playback,
    Recognition,
    Models,
    Recovery,
    Download,
}

impl Activity {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Study => "学習の認識・生成・添削中",
            Self::Generate => "教材を生成・登録中",
            Self::Connection => "接続・ChatGPT認証を確認中",
            Self::Chat => "Codexが回答を準備中",
            Self::Material => "選択した往復から教材案を作成中（未登録）",
            Self::Playback => "保存した音声を再生中",
            Self::Recognition => "音声・筆跡を文字起こし中（まだ送信しない）",
            Self::Models => "サーバーからモデル・effort候補を取得中",
            Self::Recovery => "実行結果を再取得中（再生成・再登録はしない）",
            Self::Download => "NGSLの語彙一覧を取得中",
        }
    }
}

impl WordApp {
    pub(super) fn activity_label(&self) -> &'static str {
        if let Some(pending) = &self.pending {
            pending.kind.label()
        } else if self.recorder.is_some() {
            "録音中。音声・手書き入力から録音を操作できる"
        } else if self.session.is_some() {
            "学習中。学習を終了するとチャットを送信できる"
        } else if self.batch_running {
            "教材の一括生成中"
        } else if self.fatal.is_some() {
            "保存を停止中。設定画面で退避・復旧を確認"
        } else {
            "入力して送信できる"
        }
    }

    pub(super) fn connection_label(&self) -> &'static str {
        if self
            .pending
            .as_ref()
            .is_some_and(|p| p.kind == Activity::Connection)
        {
            "接続テスト：確認中"
        } else {
            match self
                .connection_check
                .as_ref()
                .filter(|(path, _)| path == self.progress.settings.codex_path.trim())
            {
                Some((_, true)) => "接続テスト：前回成功（現在の接続を保証するものではない）",
                Some((_, false)) => "接続テスト：前回失敗。設定と認証を確認して再試行",
                None => "接続テスト：未確認",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::harness_tests::fixture;
    use super::*;
    #[test]
    fn connection_status_does_not_infer_success_and_is_invalidated_by_path_change() {
        let (_, mut app, root) = fixture();
        assert!(app.connection_label().contains("未確認"));
        app.connection_check = Some((app.progress.settings.codex_path.trim().into(), true));
        assert!(app.connection_label().contains("前回成功"));
        app.progress.settings.codex_path = "different-cli".into();
        assert!(app.connection_label().contains("未確認"));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn successful_check_retains_the_checked_path_even_if_settings_change() {
        let (ctx, mut app, root) = fixture();
        let original = app.progress.settings.codex_path.trim().to_string();
        app.connection_check = Some((original.clone(), false));
        let (tx, rx) = mpsc::channel();
        app.pending = Some(Pending {
            kind: Activity::Connection,
            key: app.key(),
            rx,
            cancel: None,
        });
        assert!(app.connection_label().contains("確認中"));
        app.progress.settings.codex_path = "different-cli".into();
        tx.send(Ok(AiResult::Connection("接続成功".into())))
            .unwrap();
        app.tick(&ctx);
        assert_eq!(app.connection_check, Some((original, true)));
        assert!(app.connection_label().contains("未確認"));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
