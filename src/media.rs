use cpal::traits::{DeviceTrait,HostTrait,StreamTrait};
use std::{sync::{Arc,Mutex},time::Instant,io::Cursor,path::PathBuf};

pub struct Speaker { engine: Option<tts::Tts>, pub voices:Vec<(String,String)> }
impl Speaker {
    pub fn new()->Self {
        let engine=tts::Tts::default().ok();
        let voices=engine.as_ref().and_then(|t|t.voices().ok()).unwrap_or_default().iter()
            .map(|v|(v.id(),format!("{} ({})",v.name(),v.language()))).collect();
        Self{engine,voices}
    }
    pub fn stop(&mut self) {if let Some(t)=self.engine.as_mut(){let _=t.stop();}}
    pub fn say(&mut self,text:&str,voice_id:&str,slow:bool)->Result<(),String> {
        let t=self.engine.as_mut().ok_or("Windowsの音声合成を初期化できません。英語の音声機能を追加してください。")?;
        let voices=t.voices().map_err(|e|e.to_string())?;
        let selected=if voice_id.is_empty(){voices.iter().find(|v|v.language().to_string().to_lowercase().starts_with("en"))}
            else {voices.iter().find(|v|v.id()==voice_id)};
        let voice=selected.ok_or("英語の音声が見つかりません。Windowsに英語の音声を追加し、アプリを再起動してください。")?;
        t.set_voice(voice).map_err(|e|e.to_string())?;
        let rate=if slow {t.normal_rate()*0.8} else {t.normal_rate()};
        t.set_rate(rate.clamp(t.min_rate(),t.max_rate())).map_err(|e|e.to_string())?;
        t.speak(text,true).map_err(|e|e.to_string())?;
        Ok(())
    }
}

pub struct Recorder {
    stream:cpal::Stream,
    samples:Arc<Mutex<Vec<i16>>>,
    error:Arc<Mutex<Option<String>>>,
    sample_rate:u32,
    pub started:Instant,
}

fn append<T:Copy>(input:&[T],channels:usize,data:&Arc<Mutex<Vec<i16>>>,limit:usize,convert:impl Fn(T)->f32) {
    if let Ok(mut out)=data.lock(){
        for frame in input.chunks(channels){
            if out.len()>=limit {break;}
            let v=frame.iter().map(|s|convert(*s)).sum::<f32>()/frame.len() as f32;
            out.push((v.clamp(-1.0,1.0)*i16::MAX as f32).round() as i16);
        }
    }
}
impl Recorder {
    pub fn start()->Result<Self,String> {
        let device=cpal::default_host().default_input_device().ok_or("マイクがありません。Windowsの既定の入力デバイスを確認してください。")?;
        let supported=device.default_input_config().map_err(|e|e.to_string())?;
        let config:cpal::StreamConfig=supported.clone().into();
        let channels=config.channels as usize;
        let sample_rate=config.sample_rate.0;
        if channels==0 || sample_rate>192000 {return Err("このマイク形式には対応していません。".into());}
        let data=Arc::new(Mutex::new(Vec::new()));
        let error=Arc::new(Mutex::new(None));
        let cap=sample_rate as usize*30;
        macro_rules! stream {
            ($sample:ty,$convert:expr)=>{{
                let d=data.clone();let err=error.clone();
                device.build_input_stream(&config,move |input:&[$sample],_:&cpal::InputCallbackInfo|append(input,channels,&d,cap,$convert),move |e|{if let Ok(mut x)=err.lock(){*x=Some(e.to_string());}},None)
            }};
        }
        let stream=match supported.sample_format(){
            cpal::SampleFormat::F32=>stream!(f32,|x:f32|x),
            cpal::SampleFormat::I16=>stream!(i16,|x:i16|x as f32/32768.0),
            cpal::SampleFormat::U16=>stream!(u16,|x:u16|(x as f32-32768.0)/32768.0),
            _=>return Err("このマイクのサンプル形式には対応していません。".into()),
        }.map_err(|e|e.to_string())?;
        stream.play().map_err(|e|format!("録音を開始できません。マイクのプライバシー設定を確認してください: {e}"))?;
        Ok(Self{stream,samples:data,error,sample_rate,started:Instant::now()})
    }
    pub fn level(&self)->f32 {
        self.samples.lock().ok().map(|s|s.iter().rev().take(1000).map(|x|(*x as f32/32768.0).abs()).fold(0.0,f32::max)).unwrap_or(0.0)
    }
    pub fn finish(self)->Result<Vec<u8>,String> {
        drop(self.stream);
        if let Some(error)=self.error.lock().map_err(|_|"録音状態を読み取れません。")?.as_ref(){return Err(error.clone());}
        let samples=self.samples.lock().map_err(|_|"録音データを読み取れません。")?;
        if samples.len()<self.sample_rate as usize/4 {return Err("録音が短すぎます。".into());}
        let spec=hound::WavSpec {channels:1,sample_rate:self.sample_rate,bits_per_sample:16,sample_format:hound::SampleFormat::Int};
        let mut cursor=Cursor::new(Vec::new());
        {
            let mut writer=hound::WavWriter::new(&mut cursor,spec).map_err(|e|e.to_string())?;
            for &sample in samples.iter(){writer.write_sample(sample).map_err(|e|e.to_string())?;}
            writer.finalize().map_err(|e|e.to_string())?;
        }
        Ok(cursor.into_inner())
    }
}

#[link(name="winmm")]
extern "system" {fn PlaySoundW(sound:*const u16,module:isize,flags:u32)->i32;}

pub fn playback(wav:Vec<u8>,dir:PathBuf)->Result<(),String> {
    let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let path=dir.join(format!("playback-{stamp}.wav"));
    std::fs::write(&path,wav).map_err(|e|e.to_string())?;
    let name:Vec<u16>=path.as_os_str().to_string_lossy().encode_utf16().chain(Some(0)).collect();
    // SND_FILENAME | SND_NODEFAULT; synchronous on a dedicated worker thread.
    // `name` remains alive throughout the native call.
    let ok=unsafe{PlaySoundW(name.as_ptr(),0,0x00020002)};
    let _=std::fs::remove_file(path);
    if ok==0 {Err("録音を再生できませんでした。".into())}else{Ok(())}
}
