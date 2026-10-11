form: "WordWeave acoustic probe"
    sentence: "Input path", "input.wav"
endform
Read from file: input_path$
To Pitch (filtered autocorrelation): 0.01, 50, 800, 15, "no", 0.03, 0.09, 0.50, 0.055, 0.35, 0.14
frames = Get number of frames
writeInfoLine: "WWPITCH", tab$, "1", tab$, frames
for frame from 1 to frames
    time = Get time from frame number: frame
    f0 = Get value in frame: frame, "Hertz"
    if f0 = undefined
        appendInfoLine: fixed$(time, 9), tab$, "unvoiced"
    else
        appendInfoLine: fixed$(time, 9), tab$, fixed$(f0, 9)
    endif
endfor
