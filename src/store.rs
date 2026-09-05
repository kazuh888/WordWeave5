use crate::{model::Skill, scheduler::{Grade, Memory}};
use serde::{Deserialize,Serialize};
use std::{collections::{BTreeMap,BTreeSet}, fs::{self,File,OpenOptions}, io::Write, path::{Path,PathBuf}};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub minutes: u32,
    pub new_per_day: usize,
    pub skills: Vec<Skill>,
    pub topic: String,
    pub font_scale: f32,
    pub api_base: String,
    pub text_model: String,
    pub audio_model: String,
    pub ai_daily_limit: u32,
    pub voice_id: String,
    pub slow_speech: bool,
}
impl Default for Settings {
    fn default()->Self { Self {
        minutes:5,new_per_day:3,skills:vec![Skill::Recall,Skill::Usage],topic:"すべて".into(),font_scale:1.0,
        api_base:"https://api.openai.com/v1".into(),text_model:"gpt-4.1-mini".into(),audio_model:"whisper-1".into(),
        ai_daily_limit:10,voice_id:String::new(),slow_speech:false,
    } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Review {
    pub key: String,
    pub at: i64,
    pub date: String,
    pub grade: Grade,
    pub assisted: bool,
    pub method: String,
    pub first: bool,
    pub elapsed_days: f64,
    pub seconds: u32,
    pub self_assessed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub version: u32,
    pub settings: Settings,
    pub memories: BTreeMap<String,Memory>,
    pub reviews: Vec<Review>,
    pub suspended: BTreeSet<String>,
    pub ai_calls: BTreeMap<String,u32>,
    pub study_seconds: BTreeMap<String,u32>,
    #[serde(default)]
    pub deck_versions: BTreeMap<String,String>,
}
impl Default for Progress {
    fn default()->Self {Self{version:1,settings:Settings::default(),memories:BTreeMap::new(),reviews:Vec::new(),suspended:BTreeSet::new(),ai_calls:BTreeMap::new(),study_seconds:BTreeMap::new(),deck_versions:BTreeMap::new()}}
}
impl Progress {
    pub fn reconcile_deck(&mut self,deck:&[crate::model::Entry])->usize {
        let mut changed=0;
        for entry in deck {
            let fingerprint=entry.fingerprint();
            if self.deck_versions.get(&entry.id).is_some_and(|old|old!=&fingerprint) {
                let prefix=format!("{}:",entry.id);
                self.memories.retain(|key,_|!key.starts_with(&prefix));changed+=1;
            }
            self.deck_versions.insert(entry.id.clone(),fingerprint);
        }
        changed
    }
    pub fn validate(&self)->Result<(),String> {
        if self.version != 1 { return Err("対応していないデータ形式です。".into()); }
        if self.settings.minutes == 0 || self.settings.minutes > 30 || self.settings.new_per_day > 20 ||
            self.settings.skills.is_empty() || self.settings.ai_daily_limit > 1000 ||
            !(0.8..=1.6).contains(&self.settings.font_scale) {
            return Err("設定値が範囲外です。".into());
        }
        if self.memories.values().any(|m| !m.stability.is_finite() || m.stability <= 0.0 || m.stability > 180.0 || m.cue_chars>2 || m.last<0 || m.due<0) {
            return Err("復習データが不正です。".into());
        }
        Ok(())
    }
    pub fn record(&mut self,key:String,grade:Grade,assisted:bool,method:&str,now:i64,date:&str,seconds:u32,self_assessed:bool) {
        let m=self.memories.entry(key.clone()).or_default();
        let first=m.reviews==0;
        let elapsed_days=if first{0.0}else{now.saturating_sub(m.last).max(0) as f64 /86400.0};
        let effective=if assisted && matches!(grade,Grade::Good|Grade::Easy){Grade::Hard}else{grade};
        m.apply(effective,assisted,now);
        self.reviews.push(Review{key,at:now,date:date.into(),grade:effective,assisted,method:method.into(),first,elapsed_days,seconds,self_assessed});
    }
    pub fn observed_retention(&self, days:f64)->Option<(usize,usize)> {
        let relevant:Vec<_>=self.reviews.iter().filter(|r|r.elapsed_days>=days && !r.assisted && !r.first).collect();
        let total=relevant.len();
        if total==0 {None} else {Some((relevant.iter().filter(|r|matches!(r.grade,Grade::Good|Grade::Easy)).count(),total))}
    }
}

pub struct Storage {
    pub dir: PathBuf,
    _lock: File,
}
impl Storage {
    pub fn open()->Result<Self,String> {
        let root=std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATAが見つかりません。")?;
        Self::at(Path::new(&root).join("WordWeave5"))
    }
    pub fn at(dir:PathBuf)->Result<Self,String> {
        fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
        let lock=OpenOptions::new().create(true).truncate(false).read(true).write(true).open(dir.join("app.lock")).map_err(|e|e.to_string())?;
        fs2::FileExt::try_lock_exclusive(&lock).map_err(|_|"ほかのWordWeave 5が起動中です。閉じてから再起動してください。")?;
        Ok(Self{dir,_lock:lock})
    }
    pub fn load(&self)->Result<Progress,String> {
        let p=self.dir.join("progress.json");
        if !p.exists(){return Ok(Progress::default());}
        if fs::metadata(&p).map_err(|e|e.to_string())?.len()>100_000_000 {return Err("学習記録が大きすぎます。".into());}
        let text=fs::read_to_string(p).map_err(|e|e.to_string())?;
        let progress:Progress=serde_json::from_str(&text).map_err(|e|format!("学習記録を読めません。元ファイルは変更していません: {e}"))?;
        progress.validate()?;
        Ok(progress)
    }
    pub fn save(&self,p:&Progress)->Result<(),String> {
        p.validate()?;
        let path=self.dir.join("progress.json");
        if path.exists(){
            let backup_dir=self.dir.join("backups");
            fs::create_dir_all(&backup_dir).map_err(|e|e.to_string())?;
            let backup=backup_dir.join(format!("progress-{}.json",chrono::Local::now().format("%Y-%m-%d")));
            if !backup.exists(){fs::copy(&path,backup).map_err(|e|e.to_string())?;}
            // Keep backups: user-controlled cleanup avoids destroying recovery points.
        }
        atomic_write(&path,&serde_json::to_vec_pretty(p).map_err(|e|e.to_string())?)
    }
}

pub fn atomic_write(path:&Path,bytes:&[u8])->Result<(),String> {
    let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let temp=path.with_extension(format!("tmp-{}-{stamp}",std::process::id()));
    let result=(||{
        let mut f=OpenOptions::new().write(true).create_new(true).open(&temp)?;
        f.write_all(bytes)?;f.sync_all()?;drop(f);
        fs::rename(&temp,path)
    })();
    if result.is_err(){let _=fs::remove_file(&temp);}
    result.map_err(|e:std::io::Error|e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn progress_roundtrip_and_observed_retention() {
        let mut p=Progress::default();
        p.record("x:recall".into(),Grade::Good,false,"keyboard",0,"a",5,false);
        p.record("x:recall".into(),Grade::Good,false,"keyboard",8*86400,"b",5,false);
        assert_eq!(p.observed_retention(7.0),Some((1,1)));
        let q:Progress=serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(q.memories["x:recall"].reviews,2);
    }
    #[test] fn edited_content_resets_only_that_word() {
        let mut d=crate::model::parse_deck(crate::model::BUILTIN_DECK).unwrap();let mut p=Progress::default();
        p.reconcile_deck(&d);
        p.memories.insert(Skill::Recall.key(&d[0].id),Memory::default());
        p.memories.insert(Skill::Recall.key(&d[1].id),Memory::default());
        d[0].usage.push_str(" 改訂");assert_eq!(p.reconcile_deck(&d),1);
        assert!(!p.memories.contains_key(&Skill::Recall.key(&d[0].id)));
        assert!(p.memories.contains_key(&Skill::Recall.key(&d[1].id)));
    }
    #[test] fn arbitrary_json_is_not_a_valid_progress_backup() {
        assert!(serde_json::from_str::<Progress>("{}").is_err());
    }
    #[test] fn corrupt_progress_is_never_overwritten_by_load() {
        let dir=std::env::temp_dir().join(format!("wordweave-test-{}-{}",std::process::id(),chrono::Utc::now().timestamp_nanos_opt().unwrap()));
        let s=Storage::at(dir.clone()).unwrap();
        fs::write(dir.join("progress.json"),b"BROKEN").unwrap();
        assert!(s.load().is_err());
        assert_eq!(fs::read_to_string(dir.join("progress.json")).unwrap(),"BROKEN");
        drop(s);fs::remove_dir_all(dir).unwrap();
    }
    #[test] fn writes_replace_and_another_instance_cannot_lock() {
        let dir=std::env::temp_dir().join(format!("wordweave-write-{}-{}",std::process::id(),chrono::Utc::now().timestamp_nanos_opt().unwrap()));
        let s=Storage::at(dir.clone()).unwrap();
        assert!(Storage::at(dir.clone()).is_err());
        s.save(&Progress::default()).unwrap();
        let mut p=Progress::default();p.settings.new_per_day=1;s.save(&p).unwrap();
        assert_eq!(s.load().unwrap().settings.new_per_day,1);
        drop(s);fs::remove_dir_all(dir).unwrap();
    }
}
