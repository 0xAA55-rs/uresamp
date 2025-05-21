# URESAMP

HIFI uresamp delivers ultrasonic-fidelity text resampling via adaptive 64-bit floating-point spectral mapping, preserving Unicode 32-bit codepoint integrity with zero-phase distortion.

Resample your text by a ratio or a length using a HIFI Resampler, each character's Unicode point is treated as an audio sample point. It is guaranteed that no audio artifact will happen.

## Usage

```
uresamp <-r ratio | -l length> <some text>
```

### Examples

To resample a text into 1.5 times of length, use this command:
```
uresamp -r1.5 <some text>
```

To resample a text into 10 characters, use this command
```
uresamp -l10 <some text>
```

Running this command will get this output:
```
uresamp -r1.5 苍茫的天涯是我的爱，绵绵的青山脚下花正开。什么样的节奏是最呀最摇摆，什么样的歌声才是最开怀。
```
Output:
```
节绵茫怀爱摇正样正开山青正，茫青的爱涯茫涯茫呀的声的爱爱样开下声下山山涯爱的爱歌开样怀是爱什是爱奏青怀青山呀山我涯脚涯开正声涯的最摆最下歌怀
```

#### Tolerance

If you had specified the tolerance, the output characters would not only come from the input but also come from the resampler's original output.
```
uresamp -r1.5 -t1000 苍茫的天涯是我的爱，绵绵的青山脚下花正开。什么样的节奏是最呀最摇摆，什么样的歌声才是最开怀。
```
Output:
```
舸筳茫忒炍撇毣柡毄巃屬青氦，茬青瘵牒溵蚇涤茫嗳疆圛皂猻犕桙张下圂下岱崠漢珓磫琹櫃廷梑怡晧瀕佰散珽娹青忶青宗击峝愨浕胶漑帿歵墒淊碽更搥杈下歈忚
```