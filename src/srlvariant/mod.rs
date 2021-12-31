use std::ops::*;
use std::cmp::*;

#[derive(Debug, Clone)]
pub enum EVar {
	SVal(String),
	IVal(i64),
	FVal(f64),
	BVal(bool),
}

impl EVar {
	fn is_float(&self) -> bool {
		match *self {
			EVar::IVal(_) => false,
			EVar::BVal(_) => false,
			EVar::FVal(_) => true,
			EVar::SVal(_) => false,
		}		
	}
	
	pub fn to_float(&self) -> f64 {
		match self {
			EVar::IVal(i1) => return *i1 as f64,
			EVar::BVal(b1) => return *b1 as i64 as f64,
			EVar::FVal(f1) => return *f1,
			EVar::SVal(s1) => return s1.parse().unwrap_or(f64::NAN),
		}
	}

	pub fn to_int(&self) -> i64 {
		match self {
			EVar::IVal(i1) => return *i1,
			EVar::BVal(b1) => return *b1 as i64,
			EVar::FVal(f1) => return *f1 as i64,
			EVar::SVal(s1) => return s1.parse().unwrap_or(0),
		}
	}

	pub fn to_bool(&self) -> bool {
		match *self {
			EVar::IVal(i1) => return i1 !=0,
			EVar::BVal(b1) => return b1,
			EVar::FVal(f1) => return f1 != 0.0,
			EVar::SVal(_) => return false,
		}
	}
	
	pub fn ev_add(&self, other:& EVar) -> EVar {
		if self.is_float() || other.is_float() {
			return EVar::FVal(self.to_float()+other.to_float());
		}
		return EVar::IVal(self.to_int()+other.to_int());
	}

	pub fn ev_sub(&self, other:& EVar) -> EVar {
		if self.is_float() || other.is_float() {
			return EVar::FVal(self.to_float()-other.to_float());
		}
		return EVar::IVal(self.to_int()-other.to_int());
	}

	pub fn ev_mul(&self, other:& EVar) -> EVar {
		if self.is_float() || other.is_float() {
			return EVar::FVal(self.to_float()*other.to_float());
		}
		return EVar::IVal(self.to_int()*other.to_int());
	}

	pub fn ev_div(&self, other:& EVar) -> EVar {
		if self.is_float() || other.is_float() {
			return EVar::FVal(self.to_float()/other.to_float());
		}
		return EVar::IVal(self.to_int()/other.to_int());
	}
	// comparison operations (return a boolean)
	pub fn ev_eq(&self, other:& EVar) -> bool {
		if self.is_float() || other.is_float() {
			return self.to_float() == other.to_float();
		}
		return self.to_int()==other.to_int();
	}
	pub fn ev_infeq(&self, other:& EVar) -> bool {
		if self.is_float() || other.is_float() {
			return self.to_float() <= other.to_float();
		}
		return self.to_int()<=other.to_int();
	}
	pub fn ev_inf(&self, other:& EVar) -> bool {
		if self.is_float() || other.is_float() {
			return self.to_float() < other.to_float();
		}
		return self.to_int()<other.to_int();
	}
	pub fn ev_supeq(&self, other:& EVar) -> bool {
		if self.is_float() || other.is_float() {
			return self.to_float() >= other.to_float();
		}
		return self.to_int()>=other.to_int();
	}
	pub fn ev_sup(&self, other:& EVar) -> bool {
		if self.is_float() || other.is_float() {
			return self.to_float() > other.to_float();
		}
		return self.to_int()>other.to_int();
	}

	
	pub fn ev_lognot(&self) -> bool {
		match *self {
			EVar::IVal(i1) => i1 == 0,
			EVar::BVal(b1) => !b1,
			EVar::FVal(f1) => f1 == 0.0,
			EVar::SVal(_) => false,
		}
	}
	
	// bit-wise operations
	pub fn ev_bitnot(&self) -> i64 {
		match *self {
			EVar::IVal(i1) => return !i1,
			EVar::BVal(b1) => return !(b1 as i64),
			EVar::FVal(f1) => return !(f1 as i64),
			EVar::SVal(_) => return 0,
		}
	}
	pub fn ev_band(&self, other:& EVar) -> i64 {
		self.to_int() & other.to_int()
	}
	pub fn ev_bor(&self, other:& EVar) -> i64 {
		self.to_int() | other.to_int()
	}
	pub fn ev_bitxor(&self, other:& EVar) -> i64 {
		self.to_int() ^ other.to_int()
	}
	pub fn ev_shl(&self, other:& EVar) -> i64 {
		self.to_int()<<other.to_int()
	}
	pub fn ev_shr(&self, other:& EVar) -> i64 {
		self.to_int()>>other.to_int()
	}
	
}

impl PartialEq for EVar {
	// overload of == klept for the moment as it is slightly faster than the more compact version ev_eq
	fn eq(&self, other: &Self) -> bool {
		match *other {
			EVar::IVal(i2) => {
				match *self {
					EVar::IVal(i1) => i1 == i2,
					EVar::BVal(b1) => b1 == (i2 != 0),
					EVar::FVal(f1) => f1 == i2 as f64,
					EVar::SVal(_) => false,
				}
			}
			EVar::BVal(b2) => {
				match *self {
					EVar::IVal(i1) => (i1!=0) == b2,
					EVar::BVal(b1) => b1 == b2,
					EVar::FVal(f1) => b2 == (f1!=0.0),
					EVar::SVal(_) => false,
				}
			}
			EVar::FVal(f2) => {
				match *self {
					EVar::IVal(i1) =>  i1 as f64 == f2,
					EVar::BVal(b1) => b1 == (f2!=0.0),
					EVar::FVal(f1) => f1 == f2,
					EVar::SVal(_) => false,
				}
			},
			EVar::SVal(_) => return false,
		}
	}
}

impl Add for EVar {
	type Output = Self;
	
	fn add(self, other: Self) -> Self {
		self.ev_add(&other)	
	}
}

impl Sub for EVar {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		self.ev_sub(&other)
	}
}

impl Mul for EVar {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		self.ev_mul(&other)
	}
}

impl Div for EVar {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		self.ev_div(&other)
	}
}

impl EVar {
	pub fn sin(&self) -> EVar {
		EVar::FVal(self.to_float().sin())
	}
	pub fn cos(&self) -> EVar {
		EVar::FVal(self.to_float().cos())
	}
	pub fn tan(&self) -> EVar {
		EVar::FVal(self.to_float().tan())
	}
	pub fn exp(&self) -> EVar {
		EVar::FVal(self.to_float().exp())
	}
	pub fn ln(&self) -> EVar {
		EVar::FVal(self.to_float().ln())
	}
	pub fn log10(&self) -> EVar {
		EVar::FVal(self.to_float().log10())
	}
	pub fn sqrt(&self) -> EVar {
		EVar::FVal(self.to_float().sqrt())
	}
	pub fn cbrt(&self) -> EVar {
		EVar::FVal(self.to_float().cbrt())
	}
	pub fn pow(&self, exp:& EVar) -> EVar {
		if exp.is_float() {
			return EVar::FVal(self.to_float().powf(exp.to_float()));
		}
		else {
			let iexp=exp.to_int();
			if iexp==0 {
				return EVar::IVal(1);
			}
			else if iexp<0 || self.is_float() {
				return EVar::FVal(self.to_float().powi(iexp as i32));
			}
			return EVar::IVal(self.to_int().pow(iexp as u32));
		}
	}

	pub fn max(&self, comp:& EVar) -> EVar {
		if self.is_float() || comp.is_float() {
			let fself=self.to_float();
			let fcomp=comp.to_float();
			return EVar::FVal(if fcomp>fself {fcomp} else {fself});
		}
		else {
			let iself=self.to_int();
			let icomp=comp.to_int();
			return EVar::IVal(if icomp>iself {icomp} else {iself});
		}
	}

	pub fn min(&self, comp:& EVar) -> EVar {
		if self.is_float() || comp.is_float() {
			let fself=self.to_float();
			let fcomp=comp.to_float();
			return EVar::FVal(if fcomp<fself {fcomp} else {fself});
		}
		else {
			let iself=self.to_int();
			let icomp=comp.to_int();
			return EVar::IVal(if icomp<iself {icomp} else {iself});
		}
	}
}
