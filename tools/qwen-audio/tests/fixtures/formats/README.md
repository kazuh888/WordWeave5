# 音声形式fixture

2026-10-03。個人音声を含まない440Hz/0.2秒の合成音である。
FFmpegのlavfi sineからWAV PCM16、MP3 libmp3lame、AAC ADTS、AMR-NB libopencore_amrnb、3GP AAC、3GPP AMR-NBを生成した。
16kHz/mono、AMR-NBのみ8kHz/mono。3GP系のmoovは末尾でありシーク検査も兼ねる。
実APIは使わず、復号検証にはFFmpeg/ffprobeを必要とする。依存不足を黙ってskipしない。

境界fixtureもlavfiで作成した。`rate22050.*`は0.2秒/22050Hzで途中属性変更用、`video.3gp`は黒128×96画像/H.263と0.2秒AAC、`large-preview.aac`は40秒/48kHz/stereo、`too-long.aac`は60.1秒/8kHz/monoである。AMR-WBはRFC4867のSIDフレーム10個をテストコードで構築する。人の音声・教材・認証情報は含まない。

`stereo.aac`は0.2秒/16kHz/2chのsine。`changed-channels.3gp`は`concat:tone.aac|stereo.aac`を`-c:a copy -f 3gp`でmuxした。stsd一つだが、別実行のashowinfoで最初5framesは1ch、後半5framesは2chと確認した。複数stsd拒否では検出できない途中変化を検査する。

厳密な60秒境界はAMR-WB SIDを3000 frames（60秒）と3001 frames（60.02秒）構成して試験する。WB入り3GPは同SID10 framesを一時領域でFFmpeg copy muxして生成し、元container bytes保持・16kHz/3200 PCM framesを照合する。生成物はテスト所有領域の終了時に削除する。
