// use rexmrg::{ReadBytes, get_endian, get_reader, get_xmrg_version};
use rexmrg::read_xmrg;
use std::f64;
use std::fs::File;
use std::io;
use std::io::Read;

// structop
fn main() -> io::Result<()> {
    println!("Hello, world!");

    println!("------------ V1 --------------");
    // ********************* start v1 ******************************
    let xmrg_data = read_xmrg("xmrg0506199516z.gz").unwrap();

    let avg = average(&xmrg_data.values);
    let max = max(&xmrg_data.values);

    println!("The avg is {}", avg);
    println!("The max is {}", max);

    // ********************* end v1 ********************************

    println!("---------- V2 ---------------");
    // ********************* start v2 ******************************

    // ********************* end v2 ********************************
    println!("Fin ...");
    // io::stdout().write_all(&row1_in_mm)?;

    Ok(())
}

pub fn tester(path: &str, stop: usize) -> io::Result<()> {
    let file = File::open(path)?;

    for (i, b) in file.bytes().enumerate() {
        println!("byte {} is: {:b}", i, b.unwrap());
        if (i + 1) % 4 == 0 {
            println!("*********  INT!! *********");
        }
        if i > 0 && (i - 1) == stop {
            break;
        }
    }

    Ok(())
}

//this is gross, do something about it later
pub fn average(data: &Vec<Vec<f64>>) -> f64 {
    // let count = (data.len() * data[0].len()) as f64;
    let mut count = 0;

    // data.iter()
    //     .flatten()
    //     .map(|x| if *x < 0.0 { 0.0 } else { *x })
    //     .sum::<f64>() / count
    data.iter()
        .flatten()
        .filter(|n| **n >= 0.0)
        .inspect(|_| count += 1)
        .sum::<f64>()
        / (count as f64)
}

pub fn max(data: &Vec<Vec<f64>>) -> f64 {
    data.iter()
        .flatten()
        .filter(|n| **n >= 0.0)
        .fold(f64::MIN, |current, n| current.max(*n))
}
