# URESAMP

HIFI uresamp delivers ultrasonic-fidelity text resampling via adaptive 64-bit floating-point spectral mapping, preserving Unicode 32-bit codepoint integrity with zero-phase distortion.

Resample your text by a ratio or a length using a HIFI Resampler, each character's Unicode point is treated as an audio sample point. No audio artifact is guaranteed.

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