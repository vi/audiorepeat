# audiorepeat

Record audio snippet from microphone (based on loundness) and immediately play it back when it sound stops.

```
$ audiorepeat
ON   <--  you speak here
OFF  <--  it hears you stopping speaking and starts playing the recording back
ON
OFF
^C
```

```
$ audiorepeat --help
Usage: audiorepeat [-R <record-device>] [-P <playback-device>] [-r <sample-rate>] [-t <threshold>] [-l <block-size>] [-h <hysteresis>]

Record sound fragments from ALSA and play them back when silence is detected

Options:
  -R, --record-device
                    ALSA device to record from
  -P, --playback-device
                    ALSA device to play into
  -r, --sample-rate sample rate for recording and playing back. Format is always
                    1 channel, s16. Default 48000.
  -t, --threshold   threshold of maximum i16 sample value, default 250. Audio
                    louder than that triggers ON mode
  -l, --block-size  number of samples per block
  -h, --hysteresis  number of silent blocks to wait before considering it "OFF".
  --help            display usage information
```

A pre-built Linux x86_64 executable is uploaded to Github Releases, but it may fail to work for you.