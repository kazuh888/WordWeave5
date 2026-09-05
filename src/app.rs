use eframe::egui::{self,Color32,RichText};
use std::{collections::VecDeque,path::PathBuf,sync::mpsc::{self,Receiver,TryRecvError},time::{Duration,Instant}};
use wordweave5::{model::{self,Entry,Skill},scheduler::{self,Grade,Task},store::{self,Progress,Storage}};
use crate::{ai,ink::Ink,media::{self,Recorder,Speaker}};

#[derive(Clone,Copy,PartialEq)]
enum Page { Home,Study,Deck,Stats,Settings }
#[derive(Clone,Copy,PartialEq)]
enum Input { Keyboard,Pen,Voice }
impl Input {fn name(self)->&'static str {match self{Self::Keyboard=>"keyboard",Self::Pen=>"pen",Self::Voice=>"voice"}}}
struct Session {
    queue:VecDeque<Task>,elapsed:Duration,budget:Duration,paused:bool,completed:usize,
    credited:u32,introduced:std::collections::BTreeMap<String,Instant>,
}
enum AiResult { Text(String),Feedback(String),Played }
struct Pending { key:String,rx:Receiver<Result<AiResult,String>> }

pub struct WordApp {
    storage:Option<Storage>,fatal:Option<String>,progress:Progress,deck:Vec<Entry>,page:Page,
    session:Option<Session>,current:Option<Task>,answer:String,revealed:bool,matched:Option<bool>,
    hints:usize,input:Input,ink:Ink,speaker:Speaker,recorder:Option<Recorder>,wav:Option<Vec<u8>>,
    pending:Option<Pending>,api_key:String,feedback:String,message:String,search:String,selected:usize,
    last_frame:Instant,last_save:Instant,card_start:Instant,font_notice:String,dirty:bool,attempted:bool,
    pending_import:Option<Vec<Entry>>,
    pending_restore:Option<Progress>,
}

fn today()->String {chrono::Local::now().format("%Y-%m-%d").to_string()}
fn now()->i64 {chrono::Utc::now().timestamp()}
fn shown(s:&str)->&str {if s=="-" {"－"} else {s}}

impl WordApp {
    pub fn new(cc:&eframe::CreationContext<'_>)->Self {
        let mut font_notice=String::new();
        let mut fonts=egui::FontDefinitions::default();
        let windows=std::env::var_os("WINDIR").map(PathBuf::from).unwrap_or_else(||PathBuf::from("C:\\Windows"));
        let candidates=["meiryo.ttc","YuGothR.ttc","msgothic.ttc"];
        if let Some(bytes)=candidates.iter().find_map(|name|std::fs::read(windows.join("Fonts").join(name)).ok()) {
            fonts.font_data.insert("japanese".into(),egui::FontData::from_owned(bytes).into());
            for family in [egui::FontFamily::Proportional,egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push("japanese".into());
            }
            cc.egui_ctx.set_fonts(fonts);
        } else {font_notice="日本語フォントが見つかりません。Windowsの日本語フォントを追加してください。".into();}
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let mut style=(*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing=egui::vec2(10.0,10.0);
        style.spacing.button_padding=egui::vec2(14.0,9.0);
        style.text_styles.insert(egui::TextStyle::Body,egui::FontId::proportional(17.0));
        style.text_styles.insert(egui::TextStyle::Button,egui::FontId::proportional(16.0));
        style.visuals.selection.bg_fill=Color32::from_rgb(33,113,117);
        cc.egui_ctx.set_style(style);
        let mut fatal=None;
        let storage=match Storage::open(){Ok(s)=>Some(s),Err(e)=>{fatal=Some(e);None}};
        let mut progress=match storage.as_ref().map(Storage::load){Some(Ok(p))=>p,Some(Err(e))=>{fatal=Some(e);Progress::default()},None=>Progress::default()};
        let mut deck=match model::parse_deck(model::BUILTIN_DECK){Ok(d)=>d,Err(e)=>{fatal=Some(e);Vec::new()}};
        if let Some(s)=&storage {
            let path=s.dir.join("custom.tsv");
            if path.exists(){
                match std::fs::read_to_string(&path).map_err(|e|e.to_string()).and_then(|t|model::parse_deck(&t)) {
                    Ok(additional)=>{
                        for e in additional {if let Some(old)=deck.iter_mut().find(|x|x.id==e.id){*old=e;}else{deck.push(e);}}
                    },
                    Err(e)=>fatal=Some(format!("追加教材を読み取れません（元ファイルは変更していません）: {e}")),
                }
            }
        }
        cc.egui_ctx.set_zoom_factor(progress.settings.font_scale);
        progress.reconcile_deck(&deck);
        let api_key=std::env::var("WORDWEAVE_API_KEY").ok().or_else(||credential(&progress.settings.api_base).ok().and_then(|c|c.get_password().ok())).unwrap_or_default();
        Self{storage,fatal,progress,deck,page:Page::Home,session:None,current:None,answer:String::new(),revealed:false,matched:None,hints:0,input:Input::Keyboard,ink:Ink::default(),speaker:Speaker::new(),recorder:None,wav:None,pending:None,api_key,feedback:String::new(),message:String::new(),search:String::new(),selected:0,last_frame:Instant::now(),last_save:Instant::now(),card_start:Instant::now(),font_notice,dirty:true,attempted:false,pending_import:None,pending_restore:None}
    }
    fn persist(&mut self) {
        if self.fatal.is_some(){return;}
        if let Some(storage)=&self.storage {
            if let Err(e)=storage.save(&self.progress){
                self.fatal=Some(format!("保存に失敗しました。これ以上の学習記録は変更しません。アプリを閉じる前に、設定画面から現在の記録をエクスポートしてください。原因: {e}"));
            }else{self.dirty=false;self.last_save=Instant::now();}
        }
    }
    fn reset_answer(&mut self) {
        self.answer.clear();self.revealed=false;self.matched=None;self.hints=0;self.attempted=false;
        self.ink.clear();self.wav=None;self.feedback.clear();self.card_start=Instant::now();
        self.speaker.stop();
    }
    fn key(&self)->String {self.current.as_ref().map(|t|t.key(&self.deck)).unwrap_or_default()}
    fn start(&mut self,minutes:u32) {
        self.message.clear();
        let max_new=if minutes<=2{1}else{self.progress.settings.new_per_day};
        let queue=scheduler::make_queue(&self.deck,&self.progress,now(),&today(),max_new);
        self.session=Some(Session{queue,elapsed:Duration::ZERO,budget:Duration::from_secs(minutes as u64*60),paused:false,completed:0,credited:0,introduced:Default::default()});
        self.page=Page::Study;self.next();
    }
    fn next(&mut self) {
        self.reset_answer();
        let timed_out=self.session.as_ref().is_some_and(|s|s.elapsed>=s.budget);
        if timed_out{self.finish();return;}
        let stamp=now();
        let eligible=self.session.as_ref().and_then(|s|s.queue.iter().position(|task|{
            task.introduce || self.progress.memories.get(&task.key(&self.deck)).map_or(true,|m|m.due<=stamp)
        }));
        self.current=eligible.and_then(|pos|self.session.as_mut().and_then(|s|s.queue.remove(pos)));
        if let Some(task)=&self.current {
            if !task.introduce && matches!(task.skill,Skill::Recall|Skill::Listening){
                self.hints=self.progress.memories.get(&task.key(&self.deck)).map(|m|m.cue_chars as usize).unwrap_or(0);
            }
        }
        if self.current.is_none(){self.finish();}
    }
    fn credit_time(&mut self) {
        if let Some(s)=self.session.as_mut(){
            let whole=s.elapsed.as_secs().min(u32::MAX as u64) as u32;
            let delta=whole.saturating_sub(s.credited);
            if delta>0{*self.progress.study_seconds.entry(today()).or_default()+=delta;s.credited=whole;self.dirty=true;}
        }
    }
    fn finish(&mut self) {
        self.credit_time();
        if let Some(s)=self.session.take(){self.message=format!("今日はここまで。{}項目に回答、学習時間は{}分{}秒。",s.completed,s.elapsed.as_secs()/60,s.elapsed.as_secs()%60);}
        self.current=None;self.speaker.stop();self.page=Page::Home;self.persist();
    }
    fn grade(&mut self,grade:Grade) {
        let Some(task)=self.current.clone()else{return;};
        let key=task.key(&self.deck);
        let recently_seen=self.session.as_ref().and_then(|s|s.introduced.get(&key)).is_some_and(|t|t.elapsed()<Duration::from_secs(45));
        let assisted=self.hints>0||recently_seen;
        let self_assessed=matches!(task.skill,Skill::Usage|Skill::Sentence)||self.input!=Input::Keyboard||self.matched!=Some(true);
        let seconds=self.card_start.elapsed().as_secs().min(600) as u32;
        self.progress.record(key,grade,assisted,self.input.name(),now(),&today(),seconds,self_assessed);
        if let Some(s)=self.session.as_mut(){
            s.completed+=1;
            if grade==Grade::Again && s.budget.saturating_sub(s.elapsed)>=Duration::from_secs(120){s.queue.push_back(task);}
        }
        self.credit_time();self.dirty=true;self.persist();
        if self.fatal.is_none(){self.next();}
    }
    fn tick(&mut self,ctx:&egui::Context) {
        let delta=self.last_frame.elapsed().min(Duration::from_secs(1));self.last_frame=Instant::now();
        if self.fatal.is_none() && self.page==Page::Study && self.pending.is_none() && ctx.input(|i|i.focused) {
            if let Some(s)=self.session.as_mut(){if !s.paused{s.elapsed+=delta;}}
        }
        if self.recorder.as_ref().is_some_and(|r|r.started.elapsed()>=Duration::from_secs(30)){self.stop_recording();}
        if self.last_save.elapsed()>Duration::from_secs(20){self.credit_time();if self.dirty{self.persist();}}
        let outcome=self.pending.as_ref().map(|p|p.rx.try_recv());
        if let Some(result)=outcome {
            match result {
                Ok(r)=>{
                    let pending=self.pending.take().unwrap();
                    if pending.key==self.key(){self.message.clear();match r {
                        Ok(AiResult::Text(t))=>{self.answer=t;self.message="認識結果を確認し、誤認識があれば直してから回答してください。".into();},
                        Ok(AiResult::Feedback(t))=>{self.feedback=t;},
                        Ok(AiResult::Played)=>{},Err(e)=>self.message=e,
                    }}
                },
                Err(TryRecvError::Disconnected)=>{self.pending=None;self.message="処理が終了しましたが結果を取得できませんでした。".into();},
                Err(TryRecvError::Empty)=>{},
            }
        }
    }
    fn launch_ai(&mut self,action:u8) {
        if self.pending.is_some()||self.recorder.is_some()||self.fatal.is_some(){return;}
        let config=match ai::Config::from_settings(&self.progress.settings,&self.api_key){Ok(c)=>c,Err(e)=>{self.message=e;return;}};
        let count=self.progress.ai_calls.get(&today()).copied().unwrap_or(0);
        if count>=self.progress.settings.ai_daily_limit{self.message="本日のAI送信回数の上限に達しました。無料の自己評価は続けられます。".into();return;}
        let Some(task)=self.current.clone()else{return;};
        let entry=self.deck[task.index].clone();
        let answer=self.answer.clone();
        let png=if action==1 {match self.ink.png(){Ok(p)=>Some(p),Err(e)=>{self.message=e;return;}}}else{None};
        let wav=if action==2{match self.wav.clone(){Some(w)=>Some(w),None=>{self.message="先に録音してください。".into();return;}}}else{None};
        if action==0 && answer.trim().is_empty(){self.message="回答または自作の英文を入力してください。".into();return;}
        *self.progress.ai_calls.entry(today()).or_default()+=1;
        self.dirty=true;self.persist();if self.fatal.is_some(){return;}
        let(tx,rx)=mpsc::channel();let key=self.key();
        std::thread::spawn(move||{
            let result=match action {
                1=>config.read_ink(png.unwrap()).map(AiResult::Text),
                2=>config.transcribe(wav.unwrap()).map(AiResult::Text),
                _=>config.feedback(&entry,task.skill.label(),&answer).map(AiResult::Feedback),
            };let _=tx.send(result);
        });
        self.pending=Some(Pending{key,rx});self.message="AIに送信中…（待ち時間は学習タイマーに含めない）".into();
    }
    fn stop_recording(&mut self) {
        if let Some(r)=self.recorder.take(){match r.finish(){Ok(w)=>{self.wav=Some(w);self.message="録音した（次の問題に進むまでメモリ内で保持）。".into();},Err(e)=>self.message=e}}
    }
    fn say(&mut self,text:&str) {
        if let Err(e)=self.speaker.say(text,&self.progress.settings.voice_id,self.progress.settings.slow_speech){self.message=e;}
    }
    fn card(&mut self,ui:&mut egui::Ui,entry:&Entry) {
        ui.heading(format!("{}  ·  {}",entry.base,entry.meaning));
        ui.small(format!("{}水準（目安） / {}",entry.level,entry.tag));
        egui::Grid::new("entry-details").num_columns(2).spacing([24.0,10.0]).show(ui,|ui|{
            ui.label("社外メール");ui.label(shown(&entry.business));ui.end_row();
            ui.label("格調・文体");ui.label(shown(&entry.elevated));ui.end_row();
            ui.label("語調・意味");ui.label(&entry.register);ui.end_row();
        });
        ui.add_space(7.0);ui.label(&entry.usage);
        ui.separator();ui.label(RichText::new(entry.completed()).size(22.0));ui.label(&entry.translation);
        if ui.button("例文を聞く").clicked(){self.say(&entry.completed());}
    }
    fn home(&mut self,ui:&mut egui::Ui) {
        ui.add_space(10.0);ui.heading("5分で、使える表現を少しずつ。");
        ui.label("基本語から、自然な社外メールと表現の違いを学ぶ。");
        ui.add_space(15.0);
        let due=scheduler::make_queue(&self.deck,&self.progress,now(),&today(),0).len();
        let new_today=self.progress.reviews.iter().filter(|r|r.date==today()&&r.first).count();
        let sec=self.progress.study_seconds.get(&today()).copied().unwrap_or(0);
        ui.horizontal(|ui|{
            ui.group(|ui|{ui.label("今日の学習");ui.heading(format!("{}分 {}秒",sec/60,sec%60));});
            ui.group(|ui|{ui.label("今回の復習候補");ui.heading(format!("{due}項目"));});
            ui.group(|ui|{ui.label("今日の新規回答");ui.heading(format!("{new_today}項目"));});
        });
        ui.add_space(18.0);
        if self.session.is_some(){
            if ui.button("学習の続きから").clicked(){self.page=Page::Study;if let Some(s)=self.session.as_mut(){s.paused=false;}}
            if ui.button("現在のセッションを終了").clicked(){self.finish();}
        }else{
            let minutes=self.progress.settings.minutes;
            if ui.add_sized([230.0,50.0],egui::Button::new(format!("{minutes}分の学習を始める"))).clicked(){self.start(minutes);}
            if ui.button("今日は2分だけ").clicked(){self.start(2);}
        }
        ui.add_space(15.0);
        ui.label("期限を過ぎた復習は、今後のセッションに分けて出題する。休んでも学習記録は失われない。");
        ui.label("新しいカードは、例を確認してから別の問題を挟んで思い出す。ヒントや直後の再現は、独力の正解と区別する。");
        ui.add_space(10.0);
        let base_count=self.deck.iter().map(|e|e.base.as_str()).collect::<std::collections::BTreeSet<_>>().len();
        ui.small(format!("教材 {}項目 / 基本語 {}語。中高水準から選んだ独自教材であり、全教科書の網羅リストではない。",self.deck.len(),base_count));
        ui.small("既定の出題は「語句・綴り」「使い分け」。聞き取り・作文は設定から追加できる。");
    }
    fn study(&mut self,ui:&mut egui::Ui) {
        let Some(task)=self.current.clone()else{ui.label("ホームから学習を始めてください。");return;};
        let entry=self.deck[task.index].clone();
        let (elapsed,budget,paused)=self.session.as_ref().map(|s|(s.elapsed.as_secs(),s.budget.as_secs(),s.paused)).unwrap_or((0,300,false));
        ui.horizontal(|ui|{
            ui.heading(task.skill.label());
            let remaining=budget.saturating_sub(elapsed);
            ui.label(format!("残り {}:{:02}",remaining/60,remaining%60));
            if ui.button(if paused{"再開"}else{"一時停止"}).clicked(){if let Some(s)=self.session.as_mut(){s.paused=!paused;}}
            if ui.add_enabled(self.pending.is_none()&&self.recorder.is_none(),egui::Button::new("ここで終える")).clicked(){self.finish();}
        });
        if self.current.is_none(){return;}
        ui.add(egui::ProgressBar::new((elapsed as f32/budget.max(1) as f32).min(1.0)).show_percentage());
        if paused {ui.label("休憩中。再開するまで学習時間は増えない。");return;}
        if elapsed>=budget {ui.colored_label(Color32::from_rgb(135,90,15),"目標時間に到達。この問題を終えたら終了する。");}
        ui.separator();
        if task.introduce {
            ui.label("はじめての項目：意味と使用条件を確認する。");
            self.card(ui,&entry);
            ui.add_space(12.0);
            if ui.button("確認した・別の問題のあとで思い出す").clicked(){
                if let Some(s)=self.session.as_mut(){
                    s.introduced.insert(task.key(&self.deck),Instant::now());
                    let mut retry=task;retry.introduce=false;
                    let pos=s.queue.len().min(2);s.queue.insert(pos,retry);
                }
                self.next();
            }
            return;
        }
        match task.skill {
            Skill::Recall=>{
                if entry.accepts(&entry.base){ui.heading(format!("{} · 場面に合う表現を思い出す",entry.meaning));}
                else{ui.heading(format!("{} · {}",entry.base,entry.meaning));}
                ui.label(format!("場面：{}",entry.context));
                ui.add_space(8.0);ui.label(RichText::new(&entry.example).size(23.0));ui.label(&entry.translation);
                ui.small("空欄に入る語句を回答する。自然な別解は、照合後に自己評価できる。");
            },
            Skill::Usage=>{ui.heading(format!("{} の使い分け",entry.base));ui.label(&entry.question);ui.small("日本語の短い説明でもよい。違いを自分の言葉で思い出す。");},
            Skill::Listening=>{
                ui.heading("聞こえた語句を英語で回答する");
                ui.small("表示から答えを推測せず、音を聞いて綴る練習。");
                if ui.add_enabled(self.recorder.is_none(),egui::Button::new("語句を聞く / もう一度")).clicked(){self.say(entry.answer());}
            },
            Skill::Sentence=>{
                ui.heading("自分の文で使ってみる");ui.label(format!("表現：{} / 場面：{}",entry.answer(),entry.context));
                ui.label("この場面に合う英文を1文作る。文体を硬くすること自体を目的にしない。");
            },
        }
        if !self.revealed {
            ui.horizontal(|ui|{
                ui.add_enabled_ui(self.recorder.is_none()&&self.pending.is_none(),|ui|{
                    ui.selectable_value(&mut self.input,Input::Keyboard,"キーボード");
                    ui.selectable_value(&mut self.input,Input::Pen,"手書き");
                    ui.selectable_value(&mut self.input,Input::Voice,"音声");
                });
            });
            ui.add_enabled_ui(self.pending.is_none(),|ui|{
                if self.input==Input::Pen {self.ink.ui(ui);}
                if self.input==Input::Voice {
                    if let Some(r)=self.recorder.as_ref(){
                        ui.label(format!("録音中 {}秒 / 最大30秒",r.started.elapsed().as_secs()));
                        ui.add(egui::ProgressBar::new(r.level()).text("入力音量"));
                        if ui.button("録音を止める").clicked(){self.stop_recording();}
                    }else{
                        ui.horizontal(|ui|{
                            if ui.button("録音する（英語）").clicked(){
                                self.speaker.stop();
                                match Recorder::start(){Ok(r)=>{self.wav=None;self.recorder=Some(r);},Err(e)=>self.message=e}
                            }
                            if ui.add_enabled(self.wav.is_some(),egui::Button::new("自分の声を聞く")).clicked(){
                                if let (Some(wav),Some(storage))=(self.wav.clone(),self.storage.as_ref()){
                                    let dir=storage.dir.clone();let(tx,rx)=mpsc::channel();let key=self.key();
                                    std::thread::spawn(move||{let _=tx.send(media::playback(wav,dir).map(|_|AiResult::Played));});
                                    self.pending=Some(Pending{key,rx});
                                }
                            }
                        });
                    }
                }
                ui.add(egui::TextEdit::multiline(&mut self.answer).desired_rows(if matches!(task.skill,Skill::Usage|Skill::Sentence){3}else{2}).desired_width(f32::INFINITY).hint_text("回答 / 認識結果の修正欄"));
                ui.horizontal_wrapped(|ui|{
                    if matches!(task.skill,Skill::Recall|Skill::Listening) && ui.button("文字のヒント").clicked(){self.hints=(self.hints+1).min(entry.answer().chars().count());}
                    if self.hints>0{ui.label(format!("ヒント：{}",entry.hint(self.hints)));}
                });
                ui.add_enabled_ui(self.recorder.is_none(),|ui|{
                    ui.horizontal_wrapped(|ui|{
                        if self.input==Input::Pen && ui.button("手書き英語をAIで文字起こし（送信・有料）").clicked(){self.launch_ai(1);}
                        if self.input==Input::Voice && ui.button("録音をAIで文字起こし（送信・有料）").clicked(){self.launch_ai(2);}
                        if ui.button("回答を照合 / わからないので確認").clicked(){
                            self.attempted=!self.answer.trim().is_empty()||(self.input==Input::Pen&&!self.ink.empty())||(self.input==Input::Voice&&self.wav.is_some());
                            self.matched=if matches!(task.skill,Skill::Recall|Skill::Listening)&&!self.answer.trim().is_empty(){Some(entry.accepts(&self.answer))}else{None};
                            self.revealed=true;
                        }
                    });
                });
            });
        }else{
            match self.matched {
                Some(true)=>{ui.colored_label(Color32::from_rgb(25,115,70),"登録されている解答と一致した。");},
                Some(false)=>{ui.colored_label(Color32::from_rgb(155,92,25),"登録例とは異なる。別解の可能性を含め、意味・文法・場面を確認する。");},
                None=>{ui.label("自分の回答と、以下の解説を照合する。");},
            }
            if !self.answer.is_empty(){ui.label(format!("自分の回答：{}",self.answer));}
            if self.input==Input::Pen {ui.add_enabled_ui(false,|ui|self.ink.ui(ui));}
            self.card(ui,&entry);
            if matches!(task.skill,Skill::Usage|Skill::Sentence){ui.separator();ui.label(&entry.explanation);}
            if !self.feedback.is_empty(){ui.group(|ui|{ui.label("AIの参考コメント（学習成績は自動変更しない）");ui.label(&self.feedback);});}
            if ui.add_enabled(self.pending.is_none()&&!self.answer.trim().is_empty(),egui::Button::new("AIに使い方を確認する（送信・有料）")).clicked(){self.launch_ai(0);}
            ui.separator();
            ui.small("結果を記録：音声・手書きの認識ミスは記憶の失敗として扱わず、自分の元の回答で評価する。");
            ui.add_enabled_ui(self.pending.is_none(),|ui|{
                ui.horizontal_wrapped(|ui|{
                    if ui.button("思い出せなかった").clicked(){self.grade(Grade::Again);}
                    if ui.add_enabled(self.attempted,egui::Button::new("曖昧 / ヒントあり")).clicked(){self.grade(Grade::Hard);}
                    if ui.add_enabled(self.attempted&&self.hints==0,egui::Button::new("自力でできた")).clicked(){self.grade(Grade::Good);}
                    if ui.add_enabled(self.attempted&&self.hints==0,egui::Button::new("すぐ正確にできた")).clicked(){self.grade(Grade::Easy);}
                });
            });
            ui.small("学習直後45秒未満の再現は、復習間隔を控えめに設定する。");
        }
        if self.pending.is_some(){ui.horizontal(|ui|{ui.spinner();ui.label("処理中…");});}
        ui.small(format!("AI送信先：{} / 本日の送信試行 {} / {}回",self.progress.settings.api_base,self.progress.ai_calls.get(&today()).unwrap_or(&0),self.progress.settings.ai_daily_limit));
    }
    fn deck_page(&mut self,ui:&mut egui::Ui) {
        ui.heading("教材を調べる");
        ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("基本語・表現・日本語で検索").desired_width(450.0));
        let q=self.search.to_lowercase();
        let matches:Vec<usize>=self.deck.iter().enumerate().filter(|(_,e)|format!("{} {} {} {} {}",e.base,e.meaning,e.business,e.elevated,e.usage).to_lowercase().contains(&q)).map(|(i,_)|i).collect();
        ui.label(format!("{}項目",matches.len()));
        egui::ScrollArea::vertical().id_salt("deck-list").max_height(180.0).show(ui,|ui|{
            for &index in &matches {let e=&self.deck[index];if ui.selectable_label(self.selected==index,format!("{}   {}   [{}]",e.base,e.meaning,e.tag)).clicked(){self.selected=index;}}
        });
        if let Some(e)=self.deck.get(self.selected).cloned(){
            ui.separator();self.card(ui,&e);ui.label(&e.question);ui.label(&e.explanation);
            let mut suspended=self.progress.suspended.contains(&e.id);
            if ui.checkbox(&mut suspended,"この項目を学習対象から外す（記録は保持）").changed(){
                if suspended{self.progress.suspended.insert(e.id.clone());}else{self.progress.suspended.remove(&e.id);}
                self.dirty=true;self.persist();
            }
        }
    }
    fn stats(&mut self,ui:&mut egui::Ui) {
        ui.heading("覚えた感覚と、後日の再現を分けて見る");
        ui.label(format!("記録した回答：{}回",self.progress.reviews.len()));
        egui::Grid::new("retention").striped(true).show(ui,|ui|{
            ui.strong("前回学習からの間隔");ui.strong("ヒントなしの自己評価・照合結果");ui.end_row();
            for days in [7.0,30.0]{
                ui.label(format!("{days:.0}日以上"));
                ui.label(match self.progress.observed_retention(days){Some((ok,total))=>format!("{ok}/{total}回 ({:.0}%)",100.0*ok as f64/total as f64),None=>"まだ記録がない".into()});ui.end_row();
            }
        });
        ui.small("これは通常の復習記録であり、無作為抽出した能力テストではない。自己評価も含む。7日以上には30日以上も含まれる。");
        ui.separator();
        egui::Grid::new("by-skill").striped(true).show(ui,|ui|{
            ui.strong("練習の種類");ui.strong("回答回数");ui.strong("復習を始めた項目");ui.end_row();
            for skill in Skill::ALL {
                let suffix=format!(":{}",skill.code());
                ui.label(skill.label());ui.label(self.progress.reviews.iter().filter(|r|r.key.ends_with(&suffix)).count().to_string());
                ui.label(self.progress.memories.keys().filter(|k|k.ends_with(&suffix)).count().to_string());ui.end_row();
            }
        });
        ui.separator();ui.label("直近7日の学習時間");
        for day in (0..7).rev(){
            let date=(chrono::Local::now().date_naive()-chrono::Duration::days(day)).format("%Y-%m-%d").to_string();
            let sec=self.progress.study_seconds.get(&date).copied().unwrap_or(0);
            ui.horizontal(|ui|{ui.label(&date);ui.add(egui::ProgressBar::new((sec as f32/300.0).min(1.0)).desired_width(300.0).text(format!("{}分{}秒",sec/60,sec%60)));});
        }
        ui.small("休んだ日は0分として表示する。連続記録が途切れても、習得履歴をリセットしない。");
        ui.separator();ui.label("入力方法別の回答記録");
        for (method,label) in [("keyboard","キーボード"),("pen","手書き"),("voice","音声")] {
            let records:Vec<_>=self.progress.reviews.iter().filter(|r|r.method==method).collect();
            let independent=records.iter().filter(|r|!r.assisted&&matches!(r.grade,Grade::Good|Grade::Easy)).count();
            ui.label(format!("{label}：{}回 / ヒントなしでできた {}回",records.len(),independent));
        }
        ui.small("方式ごとに問題や難しさが異なるため、この差だけで入力方法の効果は判定できない。");
    }
    fn settings(&mut self,ui:&mut egui::Ui,ctx:&egui::Context) {
        ui.heading("学習と入力の設定");
        let before=serde_json::to_string(&self.progress.settings).unwrap_or_default();
        ui.add(egui::Slider::new(&mut self.progress.settings.minutes,1..=30).text("通常コース（分）"));
        ui.add(egui::Slider::new(&mut self.progress.settings.new_per_day,0..=20).text("新規項目の1日上限"));
        ui.small("1日5分では3項目を初期値とする。復習候補が6項目を超える日は新規を出さない。");
        ui.horizontal_wrapped(|ui|{
            for skill in Skill::ALL {
                let mut enabled=self.progress.settings.skills.contains(&skill);
                if ui.checkbox(&mut enabled,skill.label()).changed(){
                    if enabled{self.progress.settings.skills.push(skill);}else{self.progress.settings.skills.retain(|s|*s!=skill);}
                }
            }
        });
        if self.progress.settings.skills.is_empty(){self.progress.settings.skills.push(Skill::Recall);}
        let mut tags:Vec<String>=self.deck.iter().map(|e|e.tag.clone()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
        tags.insert(0,"すべて".into());
        egui::ComboBox::from_id_salt("topic").selected_text(&self.progress.settings.topic).show_ui(ui,|ui|{
            for tag in tags {ui.selectable_value(&mut self.progress.settings.topic,tag.clone(),tag);}
        });
        ui.small("出題の設定は次のセッションから反映する。");
        if ui.add(egui::Slider::new(&mut self.progress.settings.font_scale,0.8..=1.6).text("画面の拡大率")).changed(){ctx.set_zoom_factor(self.progress.settings.font_scale);}
        ui.separator();ui.heading("音声");
        let current_voice=self.speaker.voices.iter().find(|v|v.0==self.progress.settings.voice_id).map(|v|v.1.clone()).unwrap_or_else(||"英語の音声を自動選択".into());
        egui::ComboBox::from_id_salt("voice").width(440.0).selected_text(current_voice).show_ui(ui,|ui|{
            ui.selectable_value(&mut self.progress.settings.voice_id,String::new(),"英語の音声を自動選択");
            for (id,name) in &self.speaker.voices{ui.selectable_value(&mut self.progress.settings.voice_id,id.clone(),name);}
        });
        ui.checkbox(&mut self.progress.settings.slow_speech,"少しゆっくり読み上げる");
        if ui.button("音声を確認する").clicked(){self.say("We appreciate your assistance.");}
        ui.small("読み上げはWindowsの音声合成。マイクはWindowsで設定した既定の入力デバイスを使う。");
        ui.separator();ui.heading("AI連携（任意）");
        ui.label("キーなしでも学習・復習・手書き・録音再生を利用できる。");
        ui.label("初期設定はOpenAI API。ChatGPTの画面へのログインとは別に、APIキーとAPI側の利用設定が必要。");
        ui.horizontal(|ui|{
            ui.label("送信先");
            if ui.add(egui::TextEdit::singleline(&mut self.progress.settings.api_base).desired_width(420.0)).changed(){self.api_key.clear();}
        });
        ui.small("送信先を変更した場合は、対応するキーを再入力または読み込む。");
        ui.horizontal(|ui|{ui.label("添削・手書きモデル");ui.text_edit_singleline(&mut self.progress.settings.text_model);});
        ui.horizontal(|ui|{ui.label("音声認識モデル");ui.text_edit_singleline(&mut self.progress.settings.audio_model);});
        ui.horizontal(|ui|{ui.label("APIキー");ui.add(egui::TextEdit::singleline(&mut self.api_key).password(true).desired_width(430.0));});
        ui.horizontal_wrapped(|ui|{
            if ui.button("Windows資格情報に保存").clicked(){
                let result=if self.api_key.trim().is_empty(){Err("APIキーが空です。".into())}else{credential(&self.progress.settings.api_base).and_then(|c|c.set_password(self.api_key.trim()).map_err(|e|e.to_string()))};
                self.message=result.map(|_|"この送信先用のキーをWindows資格情報に保存した。".into()).unwrap_or_else(|e|e);
            }
            if ui.button("保存済みキーを読み込む").clicked(){
                match credential(&self.progress.settings.api_base).and_then(|c|c.get_password().map_err(|e|e.to_string())){Ok(k)=>{self.api_key=k;self.message="保存済みキーを読み込んだ。".into();},Err(e)=>self.message=e}
            }
            if ui.button("保存済みキーを削除").clicked(){
                let result=credential(&self.progress.settings.api_base).and_then(|c|c.delete_credential().map_err(|e|e.to_string()));
                self.api_key.clear();self.message=result.map(|_|"保存済みキーを削除した。".into()).unwrap_or_else(|e|e);
            }
        });
        ui.add(egui::Slider::new(&mut self.progress.settings.ai_daily_limit,0..=50).text("AI送信試行の1日上限"));
        ui.small("上限は回数であり金額ではない。失敗も1回に数え、自動再送しない。送信先・モデルによって料金が変わる。");
        ui.small("送信ボタンを押したときだけ通信する。手書き画像・録音・回答文のうち、その処理に必要な内容を送る。キーは教材・学習記録に含めない。");
        ui.small("互換サービスはResponses APIと音声認識APIに対応する必要がある。OpenAI以外との接続互換性は未検証。");
        ui.separator();ui.heading("教材・バックアップ");
        let idle=self.session.is_none()&&self.pending.is_none()&&self.recorder.is_none();
        ui.horizontal_wrapped(|ui|{
            if ui.button("教材をTSVに書き出す").clicked(){
                if let Some(path)=rfd::FileDialog::new().set_file_name("wordweave-deck.tsv").add_filter("TSV",&["tsv"]).save_file(){
                    self.message=store::atomic_write(&path,model::deck_text(&self.deck).as_bytes()).map(|_|"教材を書き出した。編集後は取り込みで反映できる。".into()).unwrap_or_else(|e|e);
                }
            }
            if ui.add_enabled(idle&&self.fatal.is_none(),egui::Button::new("教材TSVを取り込む")).clicked(){
                if let Some(path)=rfd::FileDialog::new().add_filter("TSV",&["tsv"]).pick_file(){
                    match read_limited(&path,8_000_000).and_then(|t|model::parse_deck(&t)){Ok(d)=>self.pending_import=Some(d),Err(e)=>self.message=e}
                }
            }
        });
        ui.small("同じIDは更新、新しいIDは追加。内容を変更した項目の復習状態は再学習から始める。学習中の取り込みはできない。");
        ui.horizontal_wrapped(|ui|{
            if ui.button("学習記録をエクスポート").clicked(){
                self.credit_time();
                if let Some(path)=rfd::FileDialog::new().set_file_name("wordweave-progress.json").add_filter("JSON",&["json"]).save_file(){
                    let result=serde_json::to_vec_pretty(&self.progress).map_err(|e|e.to_string()).and_then(|bytes|store::atomic_write(&path,&bytes));
                    self.message=result.map(|_|"学習記録を書き出した。教材は別途TSVで書き出してください。".into()).unwrap_or_else(|e|e);
                }
            }
            if ui.add_enabled(idle&&self.storage.is_some(),egui::Button::new("学習記録を復元")).clicked(){
                if let Some(path)=rfd::FileDialog::new().add_filter("JSON",&["json"]).pick_file(){
                    match read_limited(&path,100_000_000).and_then(|t|serde_json::from_str::<Progress>(&t).map_err(|e|e.to_string())).and_then(|p|{p.validate()?;Ok(p)}){
                        Ok(p)=>self.pending_restore=Some(p),Err(e)=>self.message=e,
                    }
                }
            }
        });
        if let Some(storage)=&self.storage {ui.small(format!("保存先：{}",storage.dir.display()));}
        ui.small("日ごとのバックアップは保存先のbackupsフォルダーに残る。復元前の記録も別ファイルに退避する。");
        ui.separator();
        ui.collapsing("学習方式と限界",|ui|{
            ui.label("間隔学習・想起練習・段階的ヒントを採用。復習間隔は透明な独自の計算規則であり、FSRSでも『科学的に最速と証明された方式』でもない。");
            ui.label("正解率・入力方式別の記録を確認しながら、学習量を調整する。自己評価を含むため、数値は能力の厳密な測定ではない。");
            ui.label("詳細な研究根拠・教材の選定基準は同梱のRESEARCH.mdを参照。");
        });
        if before!=serde_json::to_string(&self.progress.settings).unwrap_or_default(){self.dirty=true;}
        if ui.add_enabled(self.fatal.is_none(),egui::Button::new("設定を保存")).clicked(){self.persist();if self.fatal.is_none(){self.message="設定を保存した。".into();}}
    }
    fn import_deck(&mut self,items:Vec<Entry>)->Result<(),String> {
        if self.session.is_some()||self.pending.is_some()||self.recorder.is_some(){return Err("学習・録音・通信を終了してから取り込んでください。".into());}
        let storage=self.storage.as_ref().ok_or("保存先がありません。")?;
        let mut merged=self.deck.clone();
        for entry in items {if let Some(old)=merged.iter_mut().find(|x|x.id==entry.id){*old=entry;}else{merged.push(entry);}}
        let text=model::deck_text(&merged);model::parse_deck(&text)?;
        let path=storage.dir.join("custom.tsv");
        if path.exists(){std::fs::copy(&path,storage.dir.join(format!("custom-before-{}.tsv",chrono::Utc::now().timestamp_millis()))).map_err(|e|e.to_string())?;}
        store::atomic_write(&path,text.as_bytes())?;
        let changed=self.progress.reconcile_deck(&merged);self.deck=merged;self.dirty=true;self.persist();
        self.message=format!("教材を取り込んだ。{changed}項目の復習状態を更新した。");Ok(())
    }
    fn restore(&mut self,mut progress:Progress)->Result<(),String> {
        if self.session.is_some()||self.pending.is_some()||self.recorder.is_some(){return Err("学習・録音・通信を終了してから復元してください。".into());}
        let storage=self.storage.as_ref().ok_or("保存先がありません。")?;
        let current=storage.dir.join("progress.json");
        if current.exists(){std::fs::copy(&current,storage.dir.join(format!("progress-before-restore-{}.json",chrono::Utc::now().timestamp_millis()))).map_err(|e|e.to_string())?;}
        // Restoring an older backup must not reset today's paid-request counter.
        let used=self.progress.ai_calls.get(&today()).copied().unwrap_or(0);
        let restored=progress.ai_calls.entry(today()).or_default();*restored=(*restored).max(used);
        progress.reconcile_deck(&self.deck);storage.save(&progress)?;
        self.api_key.clear();
        self.progress=progress;self.fatal=None;self.dirty=false;self.message="学習記録を復元した。AIキーは設定画面で読み込み直してください。".into();Ok(())
    }
    fn confirmations(&mut self,ctx:&egui::Context) {
        if let Some(items)=self.pending_import.as_ref(){
            let added=items.iter().filter(|x|!self.deck.iter().any(|e|e.id==x.id)).count();
            let updated=items.iter().filter(|x|self.deck.iter().any(|e|e.id==x.id&&e.fingerprint()!=x.fingerprint())).count();
            let(mut apply,mut cancel)=(false,false);
            egui::Window::new("教材の取り込みを確認").collapsible(false).resizable(false).show(ctx,|ui|{
                ui.label(format!("追加：{added}項目 / 内容の変更：{updated}項目"));
                ui.label("変更した項目の復習状態を再学習に戻す。以前の教材は退避する。");
                ui.horizontal(|ui|{apply=ui.button("取り込む").clicked();cancel=ui.button("キャンセル").clicked();});
            });
            if apply{let items=self.pending_import.take().unwrap();if let Err(e)=self.import_deck(items){self.message=e;}}
            else if cancel{self.pending_import=None;}
        }
        if self.pending_restore.is_some(){
            let(mut apply,mut cancel)=(false,false);
            egui::Window::new("学習記録の復元を確認").collapsible(false).resizable(false).show(ctx,|ui|{
                ui.label("現在の記録を退避し、選択した記録に戻す。現在の教材と内容が異なる項目は再学習にする。");
                ui.horizontal(|ui|{apply=ui.button("復元する").clicked();cancel=ui.button("キャンセル").clicked();});
            });
            if apply{let p=self.pending_restore.take().unwrap();if let Err(e)=self.restore(p){self.message=e;}}
            else if cancel{self.pending_restore=None;}
        }
    }
}

impl eframe::App for WordApp {
    fn update(&mut self,ctx:&egui::Context,_frame:&mut eframe::Frame) {
        self.tick(ctx);
        let confirming=self.pending_import.is_some()||self.pending_restore.is_some();
        egui::TopBottomPanel::top("navigation").show(ctx,|ui|{
            ui.horizontal_wrapped(|ui|{
                ui.label(RichText::new("WordWeave 5").strong().color(Color32::from_rgb(24,103,106)).size(24.0));
                let enabled=self.pending.is_none()&&self.recorder.is_none()&&!confirming;
                ui.add_enabled_ui(enabled,|ui|{
                    for(page,title)in[(Page::Home,"ホーム"),(Page::Study,"学習"),(Page::Deck,"教材"),(Page::Stats,"記録"),(Page::Settings,"設定")]{
                        if ui.selectable_label(self.page==page,title).clicked(){self.page=page;}
                    }
                });
            });
        });
        egui::TopBottomPanel::bottom("status").show(ctx,|ui|{
            if !self.message.is_empty(){ui.label(&self.message);}
            if !self.font_notice.is_empty(){ui.colored_label(Color32::RED,&self.font_notice);}
            if let Some(error)=&self.fatal {ui.colored_label(Color32::from_rgb(165,45,30),error);}
            ui.small("ローカル学習 / AIは送信ボタンからのみ実行");
        });
        egui::CentralPanel::default().show(ctx,|ui|{
            ui.add_enabled_ui(!confirming,|ui|{
            egui::ScrollArea::vertical().id_salt(format!("page-{}",self.page as u8)).show(ui,|ui|{
                if self.fatal.is_some()&&self.page!=Page::Settings {
                    ui.heading("学習を停止している");ui.label("設定画面で記録のエクスポート・復元を確認してください。元の保存ファイルは自動で初期化しない。");return;
                }
                match self.page{Page::Home=>self.home(ui),Page::Study=>self.study(ui),Page::Deck=>self.deck_page(ui),Page::Stats=>self.stats(ui),Page::Settings=>self.settings(ui,ctx)}
            });
            });
        });
        self.confirmations(ctx);
        ctx.request_repaint_after(Duration::from_millis(200));
    }
    fn on_exit(&mut self,_gl:Option<&eframe::glow::Context>) {
        self.recorder=None;self.speaker.stop();self.credit_time();if self.dirty{self.persist();}
    }
}

fn read_limited(path:&std::path::Path,max:u64)->Result<String,String> {
    if std::fs::metadata(path).map_err(|e|e.to_string())?.len()>max{return Err("ファイルが大きすぎます。".into());}
    std::fs::read_to_string(path).map_err(|e|format!("UTF-8のファイルを読み取れません: {e}"))
}
fn credential(base:&str)->Result<keyring::Entry,String> {
    keyring::Entry::new("WordWeave5",&format!("api-key@{}",base.trim_end_matches('/'))).map_err(|e|e.to_string())
}
