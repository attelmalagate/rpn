use std::ops::*;
use std::cmp::*;

#[derive(Debug, Copy, Clone)]
pub enum EVar {
	IVal(i64),
	FVal(f64),
	BVal(bool),
}

impl PartialEq for EVar {
	fn eq(&self, other: &Self) -> bool {
		match *other {
			EVar::IVal(i2) => {
				match *self {
					EVar::IVal(i1) => i1 == i2,
					EVar::BVal(b1) => b1 == (i2 != 0),
					EVar::FVal(f1) => f1 == i2 as f64,
				}
			}
			EVar::BVal(b2) => {
				match *self {
					EVar::IVal(i1) => (i1!=0) == b2,
					EVar::BVal(b1) => b1 == b2,
					EVar::FVal(f1) => b2 == (f1!=0.0),
				}
			}
			EVar::FVal(f2) => {
				match *self {
					EVar::IVal(i1) =>  i1 as f64 == f2,
					EVar::BVal(b1) => b1 == (f2!=0.0),
					EVar::FVal(f1) => f1 == f2,
				}
			},
		}
	}
}

impl Add for EVar {
	type Output = Self;
	
	fn add(self, other: Self) -> Self {
		match other {
			EVar::IVal(i2) => {
				match self {
					EVar::IVal(i1) => return EVar::IVal(i1+i2),
					EVar::BVal(b1) => return EVar::IVal(b1 as i64+i2),
					EVar::FVal(f1) => return EVar::FVal(f1+i2 as f64),
				}
			}
			EVar::BVal(b2) => {
				match self {
					EVar::IVal(i1) => return EVar::IVal(i1+b2 as i64),
					EVar::BVal(b1) => return EVar::IVal(b1 as i64+b2 as i64),
					EVar::FVal(f1) => return EVar::FVal((b2 as i64) as f64+f1),
				}
			}
			EVar::FVal(f2) => {
				match self {
					EVar::IVal(i1) => return EVar::FVal(i1 as f64+f2),
					EVar::BVal(b1) => return EVar::FVal((b1 as i64) as f64+f2),
					EVar::FVal(f1) => return EVar::FVal(f1+f2),
				}
			},
		}
	}
}

impl Sub for EVar {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		match other {
			EVar::IVal(i2) => {
				match self {
					EVar::IVal(i1) => return EVar::IVal(i1-i2),
					EVar::BVal(b1) => return EVar::IVal(b1 as i64-i2),
					EVar::FVal(f1) => return EVar::FVal(f1-i2 as f64),
				}
			}
			EVar::BVal(b2) => {
				match self {
					EVar::IVal(i1) => return EVar::IVal(i1-b2 as i64),
					EVar::BVal(b1) => return EVar::IVal(b1 as i64-b2 as i64),
					EVar::FVal(f1) => return EVar::FVal((b2 as i64) as f64-f1),
				}
			}
			EVar::FVal(f2) => {
				match self {
					EVar::IVal(i1) => return EVar::FVal(i1 as f64-f2),
					EVar::BVal(b1) => return EVar::FVal((b1 as i64) as f64-f2),
					EVar::FVal(f1) => return EVar::FVal(f1-f2),
				}
			},
		}
	}
}

impl Mul for EVar {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		match other {
			EVar::IVal(i2) => {
				match self {
					EVar::IVal(i1) => return EVar::IVal(i1*i2),
					EVar::BVal(b1) => return EVar::IVal(b1 as i64*i2),
					EVar::FVal(f1) => return EVar::FVal(f1*i2 as f64),
				}
			}
			EVar::BVal(b2) => {
				match self {
					EVar::IVal(i1) => return EVar::IVal(i1*b2 as i64),
					EVar::BVal(b1) => return EVar::IVal(b1 as i64*b2 as i64),
					EVar::FVal(f1) => return EVar::FVal((b2 as i64) as f64*f1),
				}
			}
			EVar::FVal(f2) => {
				match self {
					EVar::IVal(i1) => return EVar::FVal(i1 as f64*f2),
					EVar::BVal(b1) => return EVar::FVal((b1 as i64) as f64*f2),
					EVar::FVal(f1) => return EVar::FVal(f1*f2),
				}
			},
		}
	}
}

impl Div for EVar {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		match other {
			EVar::IVal(i2) => {
				match self {
					EVar::IVal(i1) => return EVar::IVal(i1/i2),
					EVar::BVal(b1) => return EVar::IVal(b1 as i64/i2),
					EVar::FVal(f1) => return EVar::FVal(f1/i2 as f64),
				}
			}
			EVar::BVal(b2) => {
				match self {
					EVar::IVal(i1) => return EVar::IVal(i1/b2 as i64),
					EVar::BVal(b1) => return EVar::IVal(b1 as i64/b2 as i64),
					EVar::FVal(f1) => return EVar::FVal(f1/(b2 as i64) as f64),
				}
			}
			EVar::FVal(f2) => {
				match self {
					EVar::IVal(i1) => return EVar::FVal(i1 as f64/f2),
					EVar::BVal(b1) => return EVar::FVal((b1 as i64) as f64/f2),
					EVar::FVal(f1) => return EVar::FVal(f1/f2),
				}
			},
		}
	}
}

impl EVar {
	pub fn sin(&self) -> EVar {
		match *self {
			EVar::IVal(i) => return EVar::FVal((i as f64).sin()),
			EVar::FVal(f) => return EVar::FVal(f.sin()),
			EVar::BVal(b) => return EVar::FVal((b as i64 as f64).sin()),
		}
	}
	pub fn cos(&self) -> EVar {
		match *self {
			EVar::IVal(i) => return EVar::FVal((i as f64).cos()),
			EVar::FVal(f) => return EVar::FVal(f.cos()),
			EVar::BVal(b) => return EVar::FVal((b as i64 as f64).cos()),
		}
	}
	pub fn pow(&self, exp:& EVar) -> EVar {
		match *exp {
			EVar::IVal(iexp) => {
				match *self {
					EVar::IVal(iself) => return EVar::IVal(iself.pow(iexp as u32)),
					EVar::FVal(fself) => return EVar::FVal(fself.powi(iexp as i32)),
					EVar::BVal(bself) => return EVar::IVal((bself as i64).pow(iexp as u32)),
				}
			},
			EVar::FVal(fexp) => {
				match *self {
					EVar::IVal(iself) => return EVar::FVal((iself as f64).powf(fexp)),
					EVar::FVal(fself) => return EVar::FVal(fself.powf(fexp)),
					EVar::BVal(bself) => return EVar::FVal((bself as i64 as f64).powf(fexp)),
				}
			},
			EVar::BVal(bexp) => {
				if !bexp {
					return EVar::IVal(1);
				}
				else {
					match *self {
						EVar::IVal(iself) => return EVar::IVal(iself),
						EVar::FVal(fself) => return EVar::FVal(fself),
						EVar::BVal(bself) => return EVar::IVal(bself as i64),
					}
				}
			},
		}
	}
	pub fn max(self, comp:EVar) -> EVar {
		match comp {
			EVar::IVal(icomp) => {
				match self {
					EVar::IVal(iself) =>
						return EVar::IVal(if icomp>iself {icomp} else {iself}),
					EVar::FVal(fself) => {
						if (icomp as f64)>fself {
							return EVar::IVal(icomp);
						} 
						else {
							return EVar::FVal(fself);
						}
					},
					EVar::BVal(bself) =>
						return EVar::IVal(if icomp>(bself as i64) {icomp} else {bself as i64}),
				}
			},
			EVar::FVal(fcomp) => {
				match self {
					EVar::IVal(iself) => {
						if fcomp > (iself as f64) {
							return EVar::FVal(fcomp);
						} 
						else {
							return EVar::IVal(iself);
						}
					},
					EVar::FVal(fself) => 
						return EVar::FVal(if fcomp>fself {fcomp} else {fself}),
					EVar::BVal(bself) => 
						return EVar::FVal(if fcomp>(bself as i64 as f64) {fcomp} else {bself as i64 as f64}),
				}
			},
			EVar::BVal(bcomp) => {
				match self {
					EVar::IVal(iself) => return EVar::IVal(iself),
					EVar::FVal(fself) => {
						if (bcomp as i64 as f64)>fself {
							return EVar::IVal(bcomp as i64);
						} 
						else {
							return EVar::FVal(fself);
						}
					},
					EVar::BVal(bself) => 
						return EVar::BVal(if bcomp as i64>(bself as i64) {bcomp} else {bself}),
				}
			},
		}
	}
}
