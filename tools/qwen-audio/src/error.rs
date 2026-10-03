use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SendDisposition {
    NotSent,
    MayHaveBeenSent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidReference,
    AudioIo,
    AudioTooLarge,
    AudioEmpty,
    AudioCorrupt,
    AudioUnsupported,
    AudioTooLong,
    AudioBackendUnavailable,
    AudioDecodeTimeout,
    AudioDecodeFailed,
    AudioCleanup,
    InvalidHost,
    InvalidKey,
    CredentialRead,
    CredentialWrite,
    Busy,
    SessionNotFound,
    InvalidState,
    GuiLaunch,
    GuiDisconnected,
    Authentication,
    RateLimited,
    Provider,
    Network,
    Timeout,
    ResponseTooLarge,
    ResponseInvalid,
    ResponseJsonInvalid,
    ResponseFieldsInvalid,
    ProtocolInvalid,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeError {
    code: ErrorCode,
    sent: SendDisposition,
}
impl SafeError {
    pub fn new(code: ErrorCode, sent: SendDisposition) -> Self {
        Self { code, sent }
    }
    pub fn code(&self) -> ErrorCode {
        self.code
    }
    pub fn disposition(&self) -> SendDisposition {
        self.sent
    }
    pub fn message(&self) -> &'static str {
        use ErrorCode::*;
        match self.code {
            InvalidReference => "参照英文は空白以外を含む10,000文字以下で入力する",
            AudioIo => "音声ファイルを読み取れない",
            AudioTooLarge => "音声ファイルは6MiB以下にする",
            AudioEmpty => "音声が空である",
            AudioCorrupt => "音声の構造が壊れている、または途中で形式が変化している",
            AudioUnsupported => {
                "対応するWAV・MP3・AAC・AMR・3GP/3GPP音声（1/2チャンネル・8～48kHz）を選択する"
            }
            AudioTooLong => "音声は60秒以下にする",
            AudioBackendUnavailable => {
                "圧縮音声の検査・再生に必要なFFmpeg/ffprobeを利用できない。WAVは利用できる"
            }
            AudioDecodeTimeout => "音声の検査・復号が15秒以内に完了しなかった",
            AudioDecodeFailed => "音声の検査・復号に失敗した。対応する形式・codecを確認する",
            AudioCleanup => {
                "検査用の一時音声コピーを削除できなかった。コピーが残っている可能性がある"
            }
            InvalidHost => "選択した地域とWorkspace API HostのHTTPS URL形式を確認する",
            InvalidKey => "APIキーの形式を確認する。接続先を変更する場合は新たにキーを入力する",
            CredentialRead => "保存済み接続設定を読み取れない",
            CredentialWrite => "接続設定を保存できない。旧設定を保持した",
            Busy => "進行中の評価がある",
            SessionNotFound => "セッションが見つからない",
            InvalidState => "現在の状態では操作できない",
            GuiLaunch => "確認画面を起動できない",
            GuiDisconnected => "確認画面との接続が切れた",
            Authentication => "認証に失敗した。接続設定を確認する",
            RateLimited => "APIの利用制限に達した",
            Provider => "APIが要求を受理できなかった",
            Network => "通信に失敗した",
            Timeout => "評価が180秒以内に完了しなかった",
            ResponseTooLarge => "API応答が上限を超えた",
            ResponseInvalid => {
                "AIの評価結果を読み取れなかった。原因は特定できず、音声の良し悪しや例文との一致は判断できない"
            }
            ResponseJsonInvalid => {
                "AIの評価結果を読み取れなかった。結果のJSON形式を確認できず、音声の良し悪しや例文との一致は判断できない"
            }
            ResponseFieldsInvalid => {
                "AIの評価結果を読み取れなかった。結果の必須項目や内容の整合を確認できず、音声の良し悪しや例文との一致は判断できない"
            }
            ProtocolInvalid => "通信形式を確認できない",
            Cancelled => "評価を取り消した",
        }
    }
}
impl std::fmt::Display for SafeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}
impl std::error::Error for SafeError {}
impl Serialize for SafeError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut x = s.serialize_struct("SafeError", 3)?;
        x.serialize_field("code", &self.code)?;
        x.serialize_field("message", self.message())?;
        x.serialize_field("send_disposition", &self.sent)?;
        x.end()
    }
}
impl<'de> Deserialize<'de> for SafeError {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            code: ErrorCode,
            message: String,
            send_disposition: SendDisposition,
        }
        let x = Wire::deserialize(d)?;
        let e = Self::new(x.code, x.send_disposition);
        if e.message() != x.message {
            return Err(serde::de::Error::custom("invalid safe message"));
        }
        Ok(e)
    }
}
pub(crate) fn err(code: ErrorCode) -> SafeError {
    SafeError::new(code, SendDisposition::NotSent)
}
pub(crate) fn sent_err(code: ErrorCode) -> SafeError {
    SafeError::new(code, SendDisposition::MayHaveBeenSent)
}
