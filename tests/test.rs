use rpn;
use crate::rpn::srlvariant::*;
use crate::rpn::*;
use crate::rpn::eval::*;
use std::time::Instant;

struct RpnTest {
	ctx:u32,
	expr:&'static str,
	prec:f64,
	expected:EVar,
}
// 20211227 - perf tests
// 1_000 times 86 test expression in 2.27 seconds, that is to say 26.3 µs/expression
// rig: AMD Ryzen 7 3700X 8-Core Processor 3.60 GHz, 64 GB, Win 10 Pro 20H2, ASUS ROG Strix X570-I mini ITX
const TESTS:[&'static RpnTest;86]=[
	&RpnTest{ctx:0, expr:"pi", prec:0.0, expected:EVar::FVal(std::f64::consts::PI)},
	&RpnTest{ctx:0, expr:"π", prec:0.0, expected:EVar::FVal(std::f64::consts::PI)},
	&RpnTest{ctx:0, expr:"phi", prec:0.0, expected:EVar::FVal(1.618_033_988_749_894_848_204_586)},
	&RpnTest{ctx:0, expr:"Φ", prec:0.0, expected:EVar::FVal(1.618_033_988_749_894_848_204_586)},
	&RpnTest{ctx:0, expr:"rho", prec:0.0, expected:EVar::FVal(1.324_717_957_244_746_025_960_908)},
	&RpnTest{ctx:0, expr:"ρ", prec:0.0, expected:EVar::FVal(1.324_717_957_244_746_025_960_908)},
	&RpnTest{ctx:0, expr:"e", prec:0.0, expected:EVar::FVal(std::f64::consts::E)},
	&RpnTest{ctx:0, expr:"e*ρ*Φ*π", prec:0.0, expected:EVar::FVal(18.304396652610638)},
	&RpnTest{ctx:0, expr:"me*c*c", prec:0.0, expected:EVar::FVal(8.187_105_776_823_886e-14)},
	&RpnTest{ctx:0, expr:"mn*c*c", prec:1e-14, expected:EVar::FVal(1.505_349_762_872_151e-10)},
	&RpnTest{ctx:0, expr:"mp*c*c", prec:0.0, expected:EVar::FVal(1.503_277_615_985_125_6e-10)},
	&RpnTest{ctx:0, expr:"true", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"false", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"0xFF", prec:0.0, expected:EVar::IVal(255)},
	&RpnTest{ctx:0, expr:"0xFF-0xff", prec:0.0, expected:EVar::IVal(0)},
	&RpnTest{ctx:0, expr:"max(!1+4,2,3*4)", prec:0.0, expected:EVar::IVal(12)},
	&RpnTest{ctx:0, expr:"max(!1+4,2,3)", prec:0.0, expected:EVar::IVal(4)},
	&RpnTest{ctx:0, expr:"max(1,2,3)", prec:0.0, expected:EVar::IVal(3)},
	&RpnTest{ctx:0, expr:"min(1,2,3)", prec:0.0, expected:EVar::IVal(1)},
	&RpnTest{ctx:0, expr:"max(1,2,3.)", prec:0.0, expected:EVar::FVal(3.)},
	&RpnTest{ctx:0, expr:"min(1.,2,3)", prec:0.0, expected:EVar::FVal(1.)},
	&RpnTest{ctx:0, expr:"max(-1,2,3.)", prec:0.0, expected:EVar::FVal(3.)},
	&RpnTest{ctx:0, expr:"min(-1.,2,3)", prec:0.0, expected:EVar::FVal(-1.)},
	&RpnTest{ctx:0, expr:"1-max((true == 1/2.),(-1),2,3,4,max(5,sin(pi)))", prec:0.0, expected:EVar::FVal(-4.)},
	&RpnTest{ctx:0, expr:"cos(pi)", prec:0.0, expected:EVar::FVal(-1.)},
	&RpnTest{ctx:0, expr:"avg(1,2,1)", prec:0.0, expected:EVar::FVal(1.3333333333333333)},
	&RpnTest{ctx:0, expr:"pow(1,0)", prec:0.0, expected:EVar::IVal(1)},
	&RpnTest{ctx:0, expr:"pow(pi,0)", prec:0.0, expected:EVar::IVal(1)},
	&RpnTest{ctx:0, expr:"pow(1,0.0)", prec:0.0, expected:EVar::FVal(1.0)},
	&RpnTest{ctx:0, expr:"pow(2,4)", prec:0.0, expected:EVar::IVal(16)},
	&RpnTest{ctx:0, expr:"pow(2,-4)", prec:0.0, expected:EVar::FVal(0.0625)},
	&RpnTest{ctx:0, expr:"pow(pi,3.5)", prec:0.0, expected:EVar::FVal(54.95719450423931)},
	&RpnTest{ctx:0, expr:"pow((1/e),1.95)", prec:0.0, expected:EVar::FVal(0.1422740715865136)},
	&RpnTest{ctx:0, expr:"pow(0,0)", prec:0.0, expected:EVar::IVal(1)},	
	&RpnTest{ctx:0, expr:"pow(0,1.95)", prec:0.0, expected:EVar::FVal(0.)},
	&RpnTest{ctx:0, expr:"pow(0,3)", prec:0.0, expected:EVar::IVal(0)},
	&RpnTest{ctx:0, expr:"!true", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"!false", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"!pi", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"!0", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"!0.0", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"!1000", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"false == !true", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"false == true", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"true == true", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"pi == true", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"pi == 1", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"8 == 4*2", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"false != true", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"true != true", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"pi != true", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"pi != 1", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"8 != 4*2", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"!(false == true)", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"!(true == true)", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"!(pi == true)", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"!(pi == 1)", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"!(8 == 4*2)", prec:0.0, expected:EVar::BVal(false)},
	&RpnTest{ctx:0, expr:"sqrt(16)", prec:0.0, expected:EVar::FVal(4.)},
	&RpnTest{ctx:0, expr:"sqrt(c*c)", prec:0.0, expected:EVar::FVal(299_792_458.0)},
	&RpnTest{ctx:0, expr:"cbrt(8.0)", prec:0.0, expected:EVar::FVal(2.0)},
	&RpnTest{ctx:0, expr:"cbrt(c*c*c)", prec:0.0, expected:EVar::FVal(299_792_458.0)},
	&RpnTest{ctx:0, expr:"ln(exp(1.0))", prec:0.0, expected:EVar::FVal(1.0)},
	&RpnTest{ctx:0, expr:"log10(1000.)", prec:0.0, expected:EVar::FVal(3.0)},
	&RpnTest{ctx:0, expr:"tan(0)", prec:0.0, expected:EVar::FVal(0.)},
	&RpnTest{ctx:0, expr:"tan(pi)", prec:1e-15, expected:EVar::FVal(0.)},
	&RpnTest{ctx:0, expr:"~0", prec:0.0, expected:EVar::IVal(-1)},
	&RpnTest{ctx:0, expr:"~8 & 0xffff", prec:0.0, expected:EVar::IVal(0xfff7)},
	&RpnTest{ctx:0, expr:"5|2", prec:0.0, expected:EVar::IVal(7)},
	&RpnTest{ctx:0, expr:"5>2", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"5*5<200.", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"5>=5.0", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"5*5<=25", prec:0.0, expected:EVar::BVal(true)},
	&RpnTest{ctx:0, expr:"2 << 4", prec:0.0, expected:EVar::IVal(32)},
	&RpnTest{ctx:0, expr:"8 >> 3", prec:0.0, expected:EVar::IVal(1)},
	&RpnTest{ctx:0, expr:"16.>> 3", prec:0.0, expected:EVar::IVal(2)},
	&RpnTest{ctx:0, expr:"5^1", prec:0.0, expected:EVar::IVal(4)},
	&RpnTest{ctx:0, expr:"5^2", prec:0.0, expected:EVar::IVal(7)},
	&RpnTest{ctx:0, expr:"1-3+6-24+2", prec:0.0, expected:EVar::IVal(-18)},
	&RpnTest{ctx:0, expr:"1-3-6", prec:0.0, expected:EVar::IVal(-8)},
	// golden ratio
	&RpnTest{ctx:0, expr:"(1+sqrt(5))/2", prec:0.0, expected:EVar::FVal(1.618_033_988_749_894_848_204_586)},
	// plastic number
	&RpnTest{ctx:0, expr:"pow((9+sqrt(69))/18,1/3.)+pow((9-sqrt(69))/18,1/3.)", prec:0.0, expected:EVar::FVal(1.324_717_957_244_746_025_960_908)},
	// test string
	&RpnTest{ctx:0, expr:"sin(\"1.57079632679489661923132169163975144\")", prec:0.0, expected:EVar::FVal(1.0)},
	// time functions
	&RpnTest{ctx:0, expr:"now()-now()", prec:0.0, expected:EVar::IVal(0)},
	// context - user-defined functions
	&RpnTest{ctx:1, expr:"var(42)", prec:0.0, expected:EVar::IVal(42)},
	&RpnTest{ctx:1, expr:"var(42.42)", prec:0.0, expected:EVar::FVal(42.42)},	
];

const USER_DEF:[FuCoOpDef;1]=[
	FuCoOpDef{fn_eval:eval_test,  params:Some(1), name:"var", prio:0, val:EVar::IVal(0)},
];


#[test]
pub fn rpn_test() {
	let context_ex=EvalContext{user_fns:& USER_DEF};
	let start = Instant::now();
	for _ in 0..1 {
		for test in TESTS {
			let mut toks=rpn::Expression::new(test.expr, if test.ctx == 1 {Some(&context_ex)} else {None});
			if let Ok(rv)=toks.eval() {
				if let Some(res)=rv {
					match test.expected {
						EVar::SVal(_) => 
							assert!(false, "eval returned wrong type for {} {:?} {:?}", test.expr, res, test.expected),
						EVar::IVal(i) => 
							if let EVar::IVal(ires)=res {
								assert!(ires == i, "test failed for {} {:?} {:?}", test.expr, res, test.expected);
							}
							else {
								assert!(false, "eval returned wrong type for {} {:?} {:?}", test.expr, res, test.expected);
							}, 
						EVar::BVal(b) => 
							if let EVar::BVal(bres)=res {
								assert!(bres == b, "test failed for {} {:?} {:?}", test.expr, res, test.expected);
							}
							else {
								assert!(false, "eval returned wrong type for {} {:?} {:?}", test.expr, res, test.expected);
							}, 
						EVar::FVal(f) => 	{
							if let EVar::FVal(fres)=res {
								assert!((fres-f).abs()<=test.prec, "test failed for {} {:?} {:?}", test.expr, res, test.expected);
							}
							else {
								assert!(false, "eval returned wrong type for {} {:?} {:?}", test.expr, res, test.expected);
							} 
						},
					}
				}
				else {
					assert!(false, "test failed for {} None {:?}", test.expr, test.expected);
				}
			}
			else {
				assert!(false, "'{}' failed", test.expr);
			}
		}
	}
	println!("elapsed for rpn_test {:#?}", start.elapsed());
}

fn eval_test(i:usize, tokens:& Vec<Token>, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	Ok(op1.val.clone())
}

/*
 * perf test on clone/copy
 * clone() takes longer than copy (approx 10%) when the Copy trait is not derived
 * otherwise, the perf is the same
 
#[derive(Debug, Clone)]
pub enum EVarEx {
	SVal(String),
	IVal(i64),
	FVal(f64),
	BVal(bool),
}

fn test_evar_clone() -> EVar {
	EVar::FVal(10.0).clone()
}
fn test_evar_copy() -> EVarEx {
	EVarEx::FVal(10.0).clone()
}

#[test]
pub fn rpn_test_evar() {
	let mut start = Instant::now();
	for _ in 0..1_000_000 {
		let v=test_evar_clone();
	}
	println!("elapsed for clone {:#?}", start.elapsed());
	start = Instant::now();
	for _ in 0..1_000_000 {
		let v=test_evar_copy();
	}
	println!("elapsed for copy {:#?}", start.elapsed());

}
 */