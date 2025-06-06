use std::{
    cmp::max,
    env::args,
    process::ExitCode,
};

use resampler::Resampler;

#[allow(dead_code)]
enum Lang {
    En,
    Zh
}

#[cfg(feature = "language_zh")]
const LANG: Lang = Lang::Zh;
#[cfg(feature = "language_en")]
const LANG: Lang = Lang::En;

const MIN_FFT_SIZE: usize = 4096;

fn usage(arg0: &str) {
    match LANG {
        Lang::Zh => {
            eprintln!("用法：{arg0} <-r 比率 | -l 长度> [-t 容差] <一些文本>");
            eprintln!("使用 HIFI 重采样器按一定比率或长度对文本进行重采样，每个字符的 Unicode 点将被视为音频采样点。保证音频不失真。");
            eprintln!("容差值的作用是用来控制输出的字符是从输入的字符里面提取还是从重采样器的输出值里提取。容差值越大，越倾向于从重采样器的输出值里提取，输出的内容也越混乱。")
        }
        Lang::En => {
            eprintln!("Usage: {arg0} <-r ratio | -l length> [-t tolerance] <some text>");
            eprintln!("Resample your text by a ratio or a length using a HIFI Resampler, each character's Unicode point is treated as an audio sample point. It is guaranteed that no audio artifact will happen.");
            eprintln!("The tolerance value is used to control whether the output characters are extracted from the input characters or from the output value of the resampler. The larger the tolerance value, the more it tends to be extracted from the output value of the resampler, and the more chaotic the output content is.")
        }
    }
}

fn main() -> ExitCode {
    let arg: Vec<String> = args().collect();
    let arg0 = if arg.len() > 0 {
        &arg[0]
    } else {
        "uresamp"
    };
    if arg.len() < 3 {
        usage(arg0);
        return ExitCode::from(1);
    }
    let arg1 = &arg[1];

    let ratio: Option<&str>;
    let length: Option<&str>;
    let mut i;
    let mut tolerance = 0.0;

    match &arg1[..2] {
        "-r" => {
            let mut r = &arg1[2..];
            if r.len() > 0 {
                i = 2;
            } else if r.len() == 0 && arg.len() > 2 {
                r = &arg[2];
                i = 3;
            } else {
                match LANG {
                    Lang::En => eprintln!("Please input the ratio."),
                    Lang::Zh => eprintln!("请输入比率值。"),
                }
                usage(arg0);
                return ExitCode::from(1);
            }
            ratio = Some(r);
            length = None;
        }
        "-l" => {
            let mut l = &arg1[2..];
            if l.len() > 0 {
                i = 2;
            } else if l.len() == 0 && arg.len() > 2 {
                l = &arg[2];
                i = 3;
            } else {
                match LANG {
                    Lang::En => eprintln!("Please input the length."),
                    Lang::Zh => eprintln!("请输入长度值。"),
                }
                usage(arg0);
                return ExitCode::from(1);
            }
            ratio = None;
            length = Some(l);
        }
        _ => {
            match LANG {
                Lang::En => eprintln!("Unknown option \"{arg1}\""),
                Lang::Zh => eprintln!("未知参数： \"{arg1}\""),
            }
            usage(&arg0);
            return ExitCode::from(1);
        }
    }

    // Parse tolerance if given
    if arg.len() > i {
        let argi = &arg[i];
        let argi: Vec<char> = argi.chars().collect();
        if argi.len() >= 2 {
            match &argi[..2] {
                val if val == "-t".chars().collect::<Vec<_>>() => {
                    let argi = &arg[i];
                    let t = &argi[2..];
                    if t.len() != 0 {
                        match t.parse() {
                            Ok(t) => {
                                tolerance = t;
                                i += 1;
                            }
                            _ => (),
                        }
                    } else if t.len() == 0 && arg.len() > i + 1 {
                        match arg[i + 1].parse() {
                            Ok(t) => {
                                tolerance = t;
                                i += 2;
                            }
                            _ => (),
                        }
                    } // else the tolerance parameter is not given
                }
                // Not given.
                _ => (),
            }
        }
    }

    if arg.len() <= i {
        match LANG {
            Lang::En => eprintln!("Please provide your text to resample."),
            Lang::Zh => eprintln!("请输入你的文本内容来进行 HIFI 重采样。"),
        }
        usage(&arg0);
        return ExitCode::from(1);
    }

    let some_text = arg[i..].join(" ");
    let some_text: Vec<char> = some_text.chars().collect();
    let target_length = if let Some(ratio) = ratio {
        let ratio = match ratio.parse::<f64>() {
            Ok(ratio) => ratio,
            Err(e) => {
                match LANG {
                    Lang::En => eprintln!("Could not parse {ratio} as an number: {e}"),
                    Lang::Zh => eprintln!("无法把“{ratio}”读取为数值。{e}"),
                }
                usage(&arg0);
                return ExitCode::from(1);
            }
        };
        (some_text.len() as f64 * ratio) as usize
    } else if let Some(length) = length {
        let length = match length.parse::<usize>() {
            Ok(length) => length,
            Err(e) => {
                match LANG {
                    Lang::En => eprintln!("Could not parse {length} as an number: {e}"),
                    Lang::Zh => eprintln!("无法把“{length}”读取为数值。{e}"),
                }
                usage(&arg0);
                return ExitCode::from(1);
            }
        };
        length
    } else {
        unreachable!();
    };

    let target_length = target_length + (target_length & 1);
    let some_text = if some_text.len() & 1 != 0 {
        some_text.iter().chain(some_text.iter().last()).copied().collect()
    } else {
        some_text
    };
    let orig_length = some_text.len();

    let process_length = max(max(orig_length, target_length), MIN_FFT_SIZE);
    let mut fft_length = 1usize;
    for i in 0..usize::BITS {
        fft_length = 1 << i;
        if fft_length >= process_length {
            break;
        }
    }

    fn get_nearest(num: f32, arr: &[f32]) -> (f32, f32) {
        let mut diff = f32::MAX;
        let mut ret = 0.0;
        arr.iter().for_each(|&n|{
            let new_diff = (n - num).abs();
            if new_diff < diff {
                ret = n;
                diff = new_diff;
            }
        });
        (ret, diff)
    }

    fn fitting(curve: &[f32], template: &[f32], tolerance: f32) -> Vec<f32> {
        let mut ret = curve.to_vec();
        ret.iter_mut().for_each(|v|{
            let (val, diff) = get_nearest(*v, template);
            if diff > tolerance {
                *v = val;
            }
        });
        ret
    }
    
    let resampler = Resampler::new(fft_length);

    let mut real_numbers: Vec<f32> = some_text.iter().map(|&c|c as u32 as f32).collect();
    let avg = real_numbers.iter().sum::<f32>() / real_numbers.len() as f32;
    real_numbers.iter_mut().for_each(|n|*n -= avg);
    let result = match resampler.resample_core(&real_numbers, target_length) {
        Ok(mut result) => {
            let new_avg = result.iter().sum::<f32>() / result.len() as f32;
            result.iter_mut().for_each(|n|*n -= new_avg);
            let mut result = fitting(&result, &real_numbers, tolerance);
            result.iter_mut().for_each(|n|*n += avg);
            result.iter().map(|&n|char::from_u32(max(n as u32, 0x20))).flatten().collect::<String>()
        }
        Err(e) => {
            match LANG {
                Lang::En => eprintln!("Error occurs when performing resample: {:?}", e),
                Lang::Zh => eprintln!("重采样时重采样器发生报错：{:?}", e),
            }
            return ExitCode::from(2);
        }
    };

    println!("{}", result);
    ExitCode::from(0)
}
