
use crate::*;
use std::time::*;

// look for first operand available for evaluaton in the vector of Tokens 'tokens', from position 'istart', going backwards
// returns a tuple: index of the token found in tokens and a reference to this token
// NB side effect: the token found will be consumed (val_consumed set to true)
pub fn get_operand (istart:usize, tokens:& Vec<Token>) ->  Result<(usize, &Token), RpnError> {
	if istart==0 {
		return Err(RpnError::Exec(ExErr::GetOperandStart));
	}
	for i in (0..istart).rev() {
		if let Some(op)=tokens.get(i) {
			if op.val_consumed.get()==false /*&& 
				(op.kind==TokKind::Number || op.kind==TokKind::Constant || op.kind==TokKind::CString)*/ {
				op.val_consumed.set(true);
				return Ok((i,op));
			}
		}
	}
 	Err(RpnError::Exec(ExErr::GetOperandMissing))
}

pub fn eval_null(_:usize, _:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	Err(RpnError::Exec(ExErr::EvalNullFn))
}

pub fn eval_plus(i:usize, tokens:& Vec<Token>, nb_param:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	if nb_param == 1 { // nb_param=1 => unary operator
		return Ok(op1.val.clone());
	}
	else {
		let (_, op2) = get_operand (i_1, tokens)?;
		return Ok(op2.val.clone()+op1.val.clone());
	}
}

pub fn eval_sub(i:usize, tokens:& Vec<Token>, nb_param:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	if nb_param == 1 { // nb_param=1 => unary operator
		return Ok(EVar::IVal(0)-op1.val.clone());
	}
	else {
		let (_, op2) = get_operand (i_1, tokens)?;
		return Ok(op2.val.clone()-op1.val.clone());
	}
}

pub fn eval_mul(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(op2.val.clone()*op1.val.clone());
}

pub fn eval_div(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	match op1.val {
			EVar::IVal(i) => if i == 0 {return Err(RpnError::Exec(ExErr::EvalDiv0));}
			EVar::FVal(f) => if f == 0.0 {return Err(RpnError::Exec(ExErr::EvalDiv0));}
			EVar::BVal(b) => if b == false {return Err(RpnError::Exec(ExErr::EvalDiv0));}
			EVar::SVal(_) => return Err(RpnError::Exec(ExErr::EvalDiv0)),
	}
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(op2.val.clone()/op1.val.clone())
}

// comparison functions 
pub fn eval_eq(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal(op2.val.ev_eq(&op1.val)));
}
pub fn eval_neq(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal(!op2.val.ev_eq(&op1.val)));
}
pub fn eval_infeq(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal(op2.val.ev_infeq(&op1.val)));
}
pub fn eval_inf(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal(op2.val.ev_inf(&op1.val)));
}
pub fn eval_supeq(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal(op2.val.ev_supeq(&op1.val)));
}
pub fn eval_sup(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal(op2.val.ev_sup(&op1.val)));
}

// bit-wise operations
pub fn eval_bitnot(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(EVar::IVal(op1.val.ev_bitnot()));
}
pub fn eval_band(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::IVal(op2.val.ev_band(&op1.val)));
}
pub fn eval_bor(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::IVal(op2.val.ev_bor(&op1.val)));
}
pub fn eval_bitxor(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::IVal(op2.val.ev_bitxor(&op1.val)));
}
pub fn eval_shl(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::IVal(op2.val.ev_shl(&op1.val)));
}
pub fn eval_shr(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::IVal(op2.val.ev_shr(&op1.val)));
}

// logical operations
pub fn eval_and(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal((op2.val == (EVar::BVal(true))) && (op1.val == (EVar::BVal(true)))));
}

pub fn eval_or(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal((op2.val == (EVar::BVal(true))) || (op1.val == (EVar::BVal(true)))));
}

pub fn eval_lognot(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(EVar::BVal(op1.val.ev_lognot()));
}

pub fn eval_sin(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.sin());
}

pub fn eval_cos(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.cos());
}

pub fn eval_tan(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.tan());
}

pub fn eval_exp(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.exp());
}

pub fn eval_ln(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.ln());
}

pub fn eval_log10(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.log10());
}

pub fn eval_pow(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(op2.val.pow(&op1.val));
}

pub fn eval_sqrt(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.sqrt());
}

pub fn eval_cbrt(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.cbrt());
}

pub fn eval_max(i:usize, tokens:& Vec<Token>, nb_param:u32) -> Result<EVar, RpnError> {
	if nb_param>0 {
		let (j, opj) = get_operand (i, tokens)?;
		let mut idx=j;
		let mut rv=opj.val.clone();
		for _ in 0..nb_param -1 {
			let (k, opk) = get_operand (idx, tokens)?;
			rv=rv.max(&opk.val);
			idx=k;
		}
		return Ok(rv);
	}
	Err(RpnError::Exec(ExErr::EvalMaxParam))
}

pub fn eval_min(i:usize, tokens:& Vec<Token>, nb_param:u32) -> Result<EVar, RpnError> {
	if nb_param>0 {
		let (j, opj) = get_operand (i, tokens)?;
		let mut idx=j;
		let mut rv=opj.val.clone();
		for _ in 0..nb_param -1 {
			let (k, opk) = get_operand (idx, tokens)?;
			rv=rv.min(&opk.val);
			idx=k;
		}
		return Ok(rv);
	}
	Err(RpnError::Exec(ExErr::EvalMaxParam))
}

pub fn eval_avg(i:usize, tokens:& Vec<Token>, nb_param:u32) -> Result<EVar, RpnError> {
	if nb_param>0 {
		let (j, opj) = get_operand (i, tokens)?;
		let mut idx=j;
		let mut rv=opj.val.clone();
		for _ in 0..nb_param -1 {
			let (k, opk) = get_operand (idx, tokens)?;
			rv = rv + opk.val.clone();
			idx=k;
		}
		return Ok(rv/EVar::FVal(nb_param as f64));
	}
	Err(RpnError::Exec(ExErr::EvalAvgParam))
}

pub fn eval_now(_:usize, _:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	Ok(EVar::IVal(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
		.unwrap_or(Duration::new(0, 0)).as_secs() as i64))
}
