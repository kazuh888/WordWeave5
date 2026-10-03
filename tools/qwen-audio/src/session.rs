use crate::{error::err, ErrorCode, EvaluationResult, SafeError, SendDisposition};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SessionId(String);
impl SessionId {
    pub fn parse(value: &str) -> Result<Self, SafeError> {
        let u = uuid::Uuid::parse_str(value).map_err(|_| err(ErrorCode::SessionNotFound))?;
        if u.get_version_num() != 4 || u.to_string() != value {
            return Err(err(ErrorCode::SessionNotFound));
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for SessionId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::parse(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum SessionState {
    AwaitingUser,
    Running,
    Completed { result: EvaluationResult },
    Failed { error: SafeError },
    Cancelled { send_disposition: SendDisposition },
}
impl SessionState {
    pub(crate) fn terminal(&self) -> bool {
        !matches!(self, Self::AwaitingUser | Self::Running)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionView {
    pub session_id: SessionId,
    #[serde(flatten)]
    pub state: SessionState,
}
pub struct SessionManager {
    active: Option<SessionView>,
    cache: VecDeque<SessionView>,
}
impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
impl SessionManager {
    pub fn new() -> Self {
        Self {
            active: None,
            cache: VecDeque::new(),
        }
    }
    pub fn start(&mut self) -> Result<SessionView, SafeError> {
        if self.active.is_some() {
            return Err(err(ErrorCode::Busy));
        }
        let v = SessionView {
            session_id: SessionId(uuid::Uuid::new_v4().to_string()),
            state: SessionState::AwaitingUser,
        };
        self.active = Some(v.clone());
        Ok(v)
    }
    pub fn get(&self, id: &SessionId) -> Result<SessionView, SafeError> {
        self.active
            .iter()
            .chain(self.cache.iter())
            .find(|v| &v.session_id == id)
            .cloned()
            .ok_or_else(|| err(ErrorCode::SessionNotFound))
    }
    pub fn mark_running(&mut self, id: &SessionId) -> Result<SessionView, SafeError> {
        let v = self.get(id)?;
        if !matches!(v.state, SessionState::AwaitingUser) {
            return Err(err(ErrorCode::InvalidState));
        }
        self.active.as_mut().unwrap().state = SessionState::Running;
        self.get(id)
    }
    fn commit(&mut self, id: &SessionId, state: SessionState) -> Result<SessionView, SafeError> {
        let old = self.get(id)?;
        if old.state.terminal() {
            return Ok(old);
        }
        let v = SessionView {
            session_id: id.clone(),
            state,
        };
        self.active = None;
        self.cache.push_back(v.clone());
        if self.cache.len() > 16 {
            self.cache.pop_front();
        }
        Ok(v)
    }
    pub fn complete(
        &mut self,
        id: &SessionId,
        result: EvaluationResult,
    ) -> Result<SessionView, SafeError> {
        let v = self.get(id)?;
        if v.state.terminal() {
            return Ok(v);
        }
        if !matches!(v.state, SessionState::Running) {
            return Err(err(ErrorCode::InvalidState));
        }
        self.commit(id, SessionState::Completed { result })
    }
    pub fn fail(&mut self, id: &SessionId, error: SafeError) -> Result<SessionView, SafeError> {
        self.commit(id, SessionState::Failed { error })
    }
    pub fn cancel(&mut self, id: &SessionId) -> Result<SessionView, SafeError> {
        let v = self.get(id)?;
        let sent = if matches!(v.state, SessionState::Running) {
            SendDisposition::MayHaveBeenSent
        } else {
            SendDisposition::NotSent
        };
        self.commit(
            id,
            SessionState::Cancelled {
                send_disposition: sent,
            },
        )
    }
}
