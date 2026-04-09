use std::cmp::{max, min};
use std::path::absolute;
use rand::prelude::*;
use serde::{Deserialize,Serialize};
#[derive(Serialize, Deserialize)]
pub struct BiasedValue{
bias: f32,
min:i32,
max:i32

}

pub fn random_range_inclusive(min:i32, max:i32) -> i32{
    let mut r = rand::rng();
    r.random_range(min..=max)

}
pub fn random_range_exclusive(min:i32, max:i32) -> i32{
    let mut r = rand::rng();
    r.random_range(min..max)

}
pub fn biased_random(bias:f32) -> f32{
   let mut r = rand::rng();
    let num: f32 = r.random_range(0.0..=1.0);

    num.powf(1.0-bias)

}

pub fn biased_random_range(min:i32,max:i32,bias:f32)->i32{
//min_val + (max_val - min_val)

    return (min as f32 + (max - min) as f32 * biased_random(bias)) as i32;

}
pub static NO_BIAS: f32 = 0.0;
pub static SOME_BIAS: f32 = 0.10;

pub static HIGHER_BIAS: f32 = 0.25;

pub static MOSTLY_BIAS: f32 = 0.50;

pub static FULLY_BIAS:f32 = 0.80;



impl BiasedValue{

    pub fn new(bias: f32,min:i32,max:i32) -> Self{
        BiasedValue{bias,min,max}
    }

pub fn validate(mut self)->  Self{

    self.min = min(self.max,self.min);
    self.max = max(self.max,self.min);
    self.bias = self.bias.max(self.bias.min(1.0));

    self
}



}

pub fn distanced_biased_random(start:i32,val:i32,max:i32,bias:f32) -> f32{

    let mut r = rand::rng();

    let hb = r.random_ratio((start - val).abs() as u32,max as u32);

    let mut bv = biased_random(bias);

    if hb {

        bv = (bv.powf(2.0));
        bv = bv.min(1.0);
    }
    return biased_random(bv);
}
pub fn distanced_biased_falloff_random(start:i32,val:i32,max:i32,bias:f32,falloff:f32) -> f32{

    let mut r = rand::rng();

    let hb = r.random_ratio(((start as f32) as i32- val).abs() as u32,max as u32);

    let mut bv = biased_random(bias);

    if hb {

        bv = (bv.powf(2.0));

        bv*=(1.0/(falloff));
        bv = bv.min(1.0);
    }

    return biased_random(bv);
}pub fn test_distribution() {
    let start = 0;
    let max = 1000;
    let samples = 10_000;

    for val in (0..=100).step_by(10) {
        let mut sum = 0.0;

        for _ in 0..samples {
            let v = distanced_biased_falloff_random(start, val, max, 0.5, 0.9);
            sum += v;
        }

        let avg = sum / samples as f32;

        println!(
            "val: {:3}, distance: {:3}, avg: {:.4}",
            val,
            (start - val).abs(),
            avg
        );
    }
}
pub fn test_distanced_biased_random() {
    let start = 0;
    let val = 30;
    let max = 100;
    let bias = 0.8;

    let iterations = 100_000;

    let mut min_v = f32::MAX;
    let mut max_v = f32::MIN;
    let mut sum = 0.0;

    let mut above_mid = 0;
    let mut below_mid = 0;

    for _ in 0..iterations {
        let v = distanced_biased_falloff_random(start, val, max, bias,1.0);

        if v < min_v { min_v = v; }
        if v > max_v { max_v = v; }

        sum += v;

        if v > 0.5 {
            above_mid += 1;
        } else {
            below_mid += 1;
        }
    }

    let avg = sum / iterations as f32;

    println!("Results over {} runs:", iterations);
    println!("min: {}", min_v);
    println!("max: {}", max_v);
    println!("avg: {}", avg);
    println!("> 0.5: {}", above_mid);
    println!("<= 0.5: {}", below_mid);
}pub fn test_distances() {
    let start = 0;
    let max = 100;
    let bias = 0.0;

    let iterations = 50_00;

    println!("dist\tavg\t>0.5\t<=0.5");

    for val in (0..=max).step_by(10) {
        let mut sum = 0.0;
        let mut above = 0;
        let mut below = 0;

        for _ in 0..iterations {
            let v = distanced_biased_falloff_random(start, val, max, bias, val as f32);

            sum += v;

            if v > 0.5 {
                above += 1;
            } else {
                below += 1;
            }
        }

        let avg = sum / iterations as f32;
        let dist = (start - val).abs();

        println!("{}\t{:.3}\t{}\t{}", dist, avg, above, below);
    }
}