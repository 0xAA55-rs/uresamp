use std::{
    cmp::max,
    env::args,
    process::ExitCode,
};

use resampler::Resampler;

fn usage(arg0: &str) {
    eprintln!("Usage: {arg0} <-r ratio | -l length> <some text>");
    eprintln!("Resample your text by a ratio or a length using a HIFI Resampler, each character's Unicode point is treated as an audio sample point. No audio artifact is guaranteed.");
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
    let i;

    match &arg1[..2] {
        "-r" => {
            let mut r = &arg1[2..];
            if r.len() > 0 {
                i = 2;
            } else if r.len() == 0 && arg.len() > 2 {
                r = &arg[2];
                i = 3;
            } else {
                eprintln!("Please input the ratio.");
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
                eprintln!("Please input the length.");
                usage(arg0);
                return ExitCode::from(1);
            }
            ratio = None;
            length = Some(l);
        }
        _ => {
            eprintln!("Unknown option \"{arg1}\"");
            usage(&arg0);
            return ExitCode::from(1);
        }
    }

    if arg.len() <= i {
        eprintln!("Please provide your text to resample.");
        usage(&arg0);
        return ExitCode::from(1);
    }

    let some_text = arg[i..].join(" ");
    let some_text: Vec<char> = some_text.chars().collect();
    let some_text = if some_text.len() & 1 != 0 {
        some_text.iter().chain(some_text.iter().last()).copied().collect()
    } else {
        some_text
    };
    let orig_length = some_text.len();
    let target_length = if let Some(ratio) = ratio {
        let ratio = match ratio.parse::<f64>() {
            Ok(ratio) => ratio,
            Err(e) => {
                eprintln!("Could not parse {ratio} as an number: {e}");
                usage(&arg0);
                return ExitCode::from(1);
            }
        };
        (some_text.len() as f64 * ratio) as usize
    } else if let Some(length) = length {
        let length = match length.parse::<usize>() {
            Ok(length) => length,
            Err(e) => {
                eprintln!("Could not parse {length} as an number: {e}");
                usage(&arg0);
                return ExitCode::from(1);
            }
        };
        length
    } else {
        unreachable!();
    };

    let process_length = max(orig_length, target_length);
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
    dbg!(&real_numbers);
    let avg = real_numbers.iter().sum::<f32>() / real_numbers.len() as f32;
    real_numbers.iter_mut().for_each(|n|*n -= avg);
    let result = match resampler.resample_core(&real_numbers, target_length) {
        Ok(mut result) => {
            let new_avg = result.iter().sum::<f32>() / result.len() as f32;
            result.iter_mut().for_each(|n|*n += avg - new_avg);
            dbg!(&result);
            result.iter().map(|&n|char::from_u32(max(n as u32, 0x20))).flatten().collect::<String>()
        }
        Err(e) => {
            eprintln!("Error occurs when performing resample: {:?}", e);
            return ExitCode::from(2);
        }
    };

    println!("{}", result);
    ExitCode::from(0)
}
