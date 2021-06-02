use std::ops::{Add, Div, Mul};

const WAD: u128 = 1 * 10_u128.pow(18);
const halfWAD: u128 = WAD / 2;

const RAY: u128 = 1 * 10_u128.pow(27);

const halfRAY: u128 = RAY / 2;

const WAD_RAY_RATIO: u128 = 1 * 10_u128.pow(9);
pub fn ray() -> u128 {
    RAY
}

pub fn wad() -> u128 {
    WAD
}

pub fn half_ray() -> u128 {
    halfRAY
}

pub fn half_wad() -> u128 {
    halfWAD
}
fn _ray_mul(a: u128, b: u128) -> u128 {
    halfRAY.add(a.mul(b)).div(RAY)
}
pub trait WadRayMath {
    fn wad_mul(&self, b: u128) -> u128;

    fn wad_div(&self, b: u128) -> u128;

    fn ray_mul(&self, b: u128) -> u128;

    fn ray_div(&self, b: u128) -> u128;

    fn ray_to_wad(&self) -> u128;

    fn wad_to_ray(&self) -> u128;

    fn ray_pow(&mut self, n: u128) -> u128;
}

impl WadRayMath for u128 {
    fn wad_mul(&self, b: u128) -> u128 {
        halfWAD.add(self.mul(b)).div(WAD)
    }

    fn wad_div(&self, b: u128) -> u128 {
        let half_b = b / 2;
        half_b.add(self.mul(WAD)).div(b)
    }

    fn ray_mul(&self, b: u128) -> u128 {
        halfRAY.add(self.mul(b)).div(RAY)
    }

    fn ray_div(&self, b: u128) -> u128 {
        let haf_b = b / 2;
        haf_b.add(self.mul(RAY)).div(b)
    }

    fn ray_to_wad(&self) -> u128 {
        let half_ratio = WAD_RAY_RATIO / 2;
        half_ratio.add(self).div(WAD_RAY_RATIO)
    }

    fn wad_to_ray(&self) -> u128 {
        self.mul(WAD_RAY_RATIO)
    }

    fn ray_pow(&mut self, mut n: u128) -> u128 {
        let mut z = if n % 2 != 0 { *self } else { RAY };

        n = n / 2;
        while n != 0 {
            *self = _ray_mul(*self, *self);
            if n % 2 != 0 {
                z = _ray_mul(z, *self);
                return z;
            }
            n = n / 2;
        }
        z
    }
}

#[cfg(test)]
mod wad_ray_tests {
    use super::*;

    #[test]
    fn wad_mul_test() {
        let res = 10_u128.pow(18).wad_mul(5);
        assert_eq!(5, res);
    }

    #[test]
    fn wad_div_test() {
        let res = 10_u128.pow(18).wad_div(5);
        assert_eq!(2 * 10_u128.pow(35), res);
    }

    #[test]
    fn ray_mul_test() {
        let res = 10_u128.pow(27).ray_mul(5);
        assert_eq!(5, res);
    }

    #[test]
    fn ray_div_test() {
        let res = 10_u128.pow(5).ray_div(5);
        assert_eq!(2 * 10_u128.pow(31), res);
    }

    #[test]
    fn ray_to_wad_test() {
        let res = 10_u128.pow(9).ray_to_wad();
        assert_eq!(1, res);
    }

    #[test]
    fn wad_to_ray_test() {
        let res = 10_u128.pow(18).ray_pow(1_000_000_000_000);
        assert_eq!(0, res);
    }
}
