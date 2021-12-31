use core::str::Chars;
use std::fmt;
use std::cell::Cell;
use std::rc::Rc;

pub mod srlvariant;
use crate::srlvariant::*;

pub mod eval;
use crate::eval::*;

#[derive(Clone, Copy)]
pub enum LexErr {
	BadStart,
	FunctionOrConstNotFound,
	FunctionOrConst,
	BadHexInit,
	BadSigBdp,
	BadSigAdp,
	BadExpSign,
	BadExpASign,
	BadExpVal,
	BadHex,
	QuoteAfterOp,
	BadOperatorShort,
	BadOperatorLong,
}
#[derive(Clone, Copy)]
pub enum ParErr {
	ParamNb,
	MatchingPar,
	ParamSep,
}
#[derive(Clone, Copy)]
pub enum ExErr {
	NotParsed,
	EvalNullFn,
	TooManyParams,
	StartWrongToken,
	WrongStackLen,
	StartOperandMissing,
	TokenNotFound,
	GetOperandMissing,
	GetOperandStart,
	EvalDiv0,
	EvalMaxParam,
	EvalAvgParam,
}

#[derive(Clone, Copy)]
pub enum RpnError {
	None,
	AnaLex(LexErr),
	Parse(ParErr),
	Exec(ExErr),
}
impl fmt::Display for RpnError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let err_msg = match *self {
			RpnError::AnaLex(_perr) => "rpn analex error",
			RpnError::Exec(_xerr) => "rpn exec error",
			RpnError::Parse(_cerr) => "rpn parsing error",
			RpnError::None => "no error",
		};
		write!(f, "{}", err_msg)
	}
}


// evaluation function prototype
type RpnEvalFn = fn(i:usize, tokens:& Vec<Token>, nb_param:u32) -> Result<EVar, RpnError>;

// catch-all structure defining the characterisitcs of functions, operators and constants
// this structure is referenced by the struct Token
pub struct FuCoOpDef {
	pub name:&'static str,
	pub prio: u32, // priority (operators only)
	pub params: Option<u32>, // nb of parameters expected (for functions with a fixed number of parameters)
	pub fn_eval:RpnEvalFn,	//evaluation function, for operators and functions
	pub val:EVar, // value (for constants only)
}

const NULL_FUCODEF:FuCoOpDef=FuCoOpDef{name:"", fn_eval:eval_null, prio:0, params:None,val:EVar::IVal(0)};

#[derive(Clone, Copy, PartialEq)]
pub enum TokKind {
	Void,
	Operator,
	Number,
	Function,
	Constant,
	CString,
	Separator,
	OPar,
	CPar,
}

#[derive(Clone)]
pub struct Token {
	kind: TokKind,
	unary: bool,
	svalue: String,
	pub val: EVar,
	nb_param: u32,
	val_consumed: Cell<bool>,
	fun_exec_done: bool,
	refdef:&'static FuCoOpDef,
}

impl Token {
	fn new() -> Token {
		Token{kind:TokKind::Void, svalue:String::from(""),	val:EVar::IVal(0), unary:false, nb_param:0, 
		val_consumed:Cell::new(false), fun_exec_done:false,refdef:&NULL_FUCODEF}
	}
	fn is_operator(&self) -> bool {
		match self.kind {
			TokKind::Operator => return true,
			_ => return false
		}
	}
	fn is_operand(&self) -> bool {
		match self.kind {
			TokKind::Number | TokKind::Constant | TokKind::CString => return true,
			_ => return false
		}
	}
	fn is_generic_sep(&self) -> bool {
		match self.kind {
			TokKind::Separator | TokKind::OPar | TokKind::CPar => return true,
			_ => return false
		}
	}
	fn is_param_sep(&self) -> bool {
		match self.kind {
			TokKind::Separator => return true,
			_ => return false
		}
	}
	fn is_opar(&self) -> bool {
		match self.kind {
			TokKind::OPar => true,
			_ => false
		}
	}
	fn is_function(&self) -> bool {
		match self.kind {
			TokKind::Function => true,
			_ => false
		}
	}
	fn is_unary(&self) -> bool {
		match self.kind {
			TokKind::Operator => {
				return self.unary;
			},
			_ => return false
		}
	}
	fn set_unary(&mut self, val:bool) -> bool {
		match self.kind {
			TokKind::Operator => {
				self.unary=val;
				if val {
					self.nb_param=1; // one operand, only for unary operators
				}
				else {
					self.nb_param=2; // not used (implicit value is two)
				}
				return val
			},
			_ => return false
		}
	}
}

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.svalue)
	}
}

pub struct ITokenVec {
	pub vec:Vec<Rc<Token>>,
}

// eval context: optional information for an Expression
// limited to user-defined functions for the moment
pub struct EvalContext {
	pub user_fns:&'static[FuCoOpDef],	
}

// main component of the rpn crate; contains:
// - a list of tokens and various status
// - methodes to tokenize, parse and evaluate an expression
pub struct Expression<'a> {
	parse_stack:Vec<Token>,
	tokens:Vec<Token>,
	itokens:ITokenVec,
	jtokens:ITokenVec,
	tokenized:bool,
	parsed:bool,
	res:Result<Option<EVar>, RpnError>,
	context:Option<&'a EvalContext>,
}

impl <'a> Expression <'a> {
	pub fn new(exp: &str, eval_context:Option<&'a EvalContext>) -> Expression<'a> {
		let mut toks=Expression {
			jtokens:ITokenVec{vec:vec![]},itokens:ITokenVec{vec:vec![]},
			tokens:vec![],parse_stack:vec![], 
			tokenized:false, parsed:false, res:Ok(None), context:eval_context};
		
		let rv=toks.parse(exp);
		if rv.is_ok() {
			println!("expression \'{}\' successully parsed as \'{}\'", exp, toks);
		}
		else {
			println!("expression \'{}\' could not be parsed with error {}", exp, rv.err().unwrap());
			toks.res = Err(rv.err().unwrap());
		}
		
		toks
	}
	
	pub fn add_context(&mut self, eval_context:&'a EvalContext){
		self.context=Some(eval_context);
	}
	
	pub fn display_parsed(&self) -> String {
		let mut msg=String::from("");
		for val in self.parse_stack.iter() {
			match val.kind {
				TokKind::Function => msg.push_str(&*format!("{}({}) ", val.svalue, val.nb_param)),
				TokKind::Operator => msg.push_str(&*format!("{}({}) ", val.svalue, 
					if val.unary {"un"} else {"bi"})),
				_ => msg.push_str(&*format!("{} ", val.svalue)),
			}
		}		
		msg
	}
	pub fn display_tokenized(&self) -> String {
		let mut msg=String::from("");
		for val in self.tokens.iter() {
			msg.push_str(&*format!("{} ", val.svalue));
		}		
		msg
	}
	pub fn result(&self) -> String {
		match &self.res {
			Ok(oval) => {
				match oval {
					Some(val) => {
						match val {
							EVar::BVal(b) => format!("{}", b),
							EVar::FVal(f) => format!("{}", f),
							EVar::IVal(i) => format!("{}", i),
							EVar::SVal(s) => format!("{}", s),
						}
					}
					None => "".to_string(),
				}
			},
			Err(e) => format!("{}", e),			
		}
	}
	// fn tokenize generates a list of token from an expression (param 'exp')
	pub fn tokenize(&mut self, exp: &str) -> Result<(), RpnError> {
		let mut exp_iter=exp.chars();
		let mut resok=true;
		let mut error=RpnError::None;
		self.tokens.clear();
		loop {
			let rv=self.get_token(&mut exp_iter);
			match rv {
				Ok(v) => {
					if v.is_none() {
						break;
					}
					else {
						self.tokens.push(v.unwrap());
						self.itokens.vec.push(Rc::new(self.tokens.last().unwrap().clone()));
					}
				},
				Err(e) => {
					println!("get_token error {}", e);
					resok=false;
					error=e;
					break;
				}
			}		
		}
		if resok {
			self.tokenized=true;
			self.res=Ok(None);
			println!("expression \'{}\' successully tokenized as {}", exp, self.display_tokenized());
			return Ok(());
		}
		println!("expression \'{}\' could not be tokenized", exp);
		self.tokenized=false;
		self.res=Err(error.clone());
		Err(error)
	}
	// fn parse creates a rpn stack from the expression passed as parameter
	// the "parsed" rpn stack is then ready for evaluation
	pub fn parse(&mut self, exp: &str) -> Result<(), RpnError>  {
		self.parse_stack.clear();
		let mut op_stack:Vec<Token> = vec![];	
		let mut resok=true;
		let mut prev_tok_kind=TokKind::Void;
		let mut error=RpnError::None;
		if !self.tokenized {
			self.tokenize(exp)?;
		}
		for rv in &self.itokens.vec {
			//self.jtokens.vec.push(rv.clone());
			self.jtokens.vec.push(Rc::clone(rv));
		}
		for rv in &self.tokens {
			//self.rtokens.vec.push(&rv);
			let mut nb_param;
			let mut tok=rv.clone();
			let mut to_stack_direct=false;
			let prev_kind=tok.kind;
			if tok.is_operand() {
				// this is an operand, to be stacked directly in the parse stack
				self.parse_stack.push(tok);
				if let Some(last_op)=op_stack.last() {
					// if in addition the last element of the operators stack is an unary operator, it is
					// moved to the parse stack
					if last_op.is_unary() {
						self.parse_stack.push(last_op.clone());
						op_stack.pop();
					}
				}
			}
			else {
				match tok.kind {
					TokKind::Function | TokKind::OPar => {
						// new token is a function or opening parenthesis, 
						// to stack directly in the operators stack
						op_stack.push(tok);
					},
					TokKind::Separator => {
						// new token is a separator of parameters (',')
						// to movve in the operator stack, after de-stacking operators to the parse stack
						if op_stack.len()==0 {
							// error, you cannot have a separator as first element in the operator stack
							error=RpnError::Parse(ParErr::ParamSep);
							break;
						}
						else {
							while let Some(last_op)=op_stack.last() {
								if last_op.is_operator() {
									self.parse_stack.push(last_op.clone());
									op_stack.pop();
								}
								else {
									break;
								}
							}
						}
						op_stack.push(tok);
					},
					TokKind::Operator => {
						// new token is an operator
						// if this is the first, stack it aside
						// else, it depends of what has been put aside before
						// if the previous token put aside is a function or opening parenthesis, or
						// if the previous token is an operator with a lower precedence, the new operator
						// is stacked aside
						if op_stack.len()==0 {
							match prev_tok_kind {
								TokKind::Void => tok.set_unary(true),
								_ => tok.set_unary(false),
							};
							op_stack.push(tok);
						}
						else {
							let last_op=op_stack.last().unwrap();
							match prev_tok_kind {
								TokKind::Void | TokKind::Separator | 
								TokKind::Operator |TokKind::OPar => tok.set_unary(true),
								_ => tok.set_unary(false),
							};
							match last_op.kind {
								TokKind::OPar |TokKind::Function => {
									to_stack_direct=true;
								},
								_ => {
									if last_op.refdef.prio>0 && last_op.refdef.prio<tok.refdef.prio && !last_op.is_unary() {
										to_stack_direct=true;
									}
								},
							};
							if !to_stack_direct {
								while let Some(last_op)=op_stack.last() {
									if last_op.is_generic_sep() {
										break;
									}
									if (last_op.refdef.prio>0 && last_op.refdef.prio >= tok.refdef.prio) || last_op.is_unary(){
										self.parse_stack.push(last_op.clone());
										op_stack.pop();
									}
									else {
										break;
									}
								}
							}
							op_stack.push(tok);									
						}
					},
					TokKind::CPar => {
						// closing parenthesis, the stacked operators are transferred to the
						// parse stack until the matching opening parenthesis is found
						// if the remaining element in the operators stack is a function, it 
						// is also tranferred to the parse stack
						let mut match_found=false;
						match prev_tok_kind {
							// the previous token was an opening parenthesis: this is a function
							// like 'fun()' with zero parameters
							TokKind::OPar => nb_param=0,
							// else the parameter count is initialized at 1 
							_ => nb_param=1,
						};
						while let Some(last_op)=op_stack.last() {
							if last_op.is_opar() {
								op_stack.pop();
								match_found=true;
								break;
							}
							else {
								if last_op.is_param_sep() {
									nb_param += 1;
								}
								else {
									// no separator nor parentheses in the parse stack
									self.parse_stack.push(last_op.clone());
								}
								op_stack.pop();
							}
						}
						if !match_found {
							// the matching opening parenthesis was not found, this is an error
							resok=false;
							error=RpnError::Parse(ParErr::MatchingPar);
							break;
						}
						if let Some(last_op)=op_stack.last() {
							if last_op.is_function() {
								let mut op=last_op.clone();
								op.nb_param=nb_param;
								if let Some(params)=op.refdef.params {
									if params != nb_param {
										resok=false;
										error=RpnError::Parse(ParErr::ParamNb);
										break;
									}
								}
								self.parse_stack.push(op);
								op_stack.pop();
							}
						}
					},
					_ => {},
				};
			}
			prev_tok_kind=prev_kind;
		}
		if resok {
			// if ok, the operators stacked is emptied into the parse stack
			while let Some(last_op)=op_stack.last() {
				if last_op.is_opar() {
					// should not happen, this is an orphan parenthesis
					resok=false;
					error=RpnError::Parse(ParErr::MatchingPar);
					break;
				}
				self.parse_stack.push(last_op.clone());
				op_stack.pop();
			}
		}
		
		if resok {
			//println!("{}", parse_stack);
			self.parsed=true;
			self.res=Ok(None);
			return Ok(());
		}
		println!("parse stack {}", self.display_parsed());
		self.parsed=false;
		self.res=Err(error.clone());
		Err(error)
	}
	pub fn eval(&mut self) -> Result<Option<EVar>, RpnError> {
		if !self.parsed {
			//self.parse(exp)?;
			if self.res.is_ok() {
				//self.compile
			}
			else {
				return Err(RpnError::Exec(ExErr::NotParsed));
			}
		}
		self.res=Ok(None);
		// reset exec flags of all tokens in the stack
		for val in &mut self.parse_stack {
			val.val_consumed.set(false);
			val.fun_exec_done = false;
		}
		// find the position of the first operator or non-zero param function
		let mut istart = self.parse_stack.iter().position(|i| (i.kind==TokKind::Operator) || 
			(i.kind==TokKind::Function && i.nb_param>0));
		// no operator nor function, meaning the stack must have one element (number, constant or zero-param function)
		// and the result for the evaluation is the value of this single element
		if istart.is_none() {
			if self.parse_stack.len() == 1 {
				if let Some(op)=self.parse_stack.get(0) {
					match op.kind {
						TokKind::Number | /*TokKind::Operator |*/ TokKind::Constant => self.res=Ok(Some(op.val.clone())),
						TokKind::Function => {
							if op.nb_param>0 {
								self.res=Err(RpnError::Exec(ExErr::TooManyParams))
							}
							else {
								let rv=(op.refdef.fn_eval)(0, & self.parse_stack, 0);
								if rv.is_ok() {
									self.res=Ok(Some(rv.ok().unwrap()));
								}
								else {
									self.res=Err(rv.err().unwrap());
								}
							}
						}
						_ => self.res=Err(RpnError::Exec(ExErr::StartWrongToken)),
					}
				}
			}
			else {
				self.res=Err(RpnError::Exec(ExErr::WrongStackLen));			
			}
		}
		// impossible (that would mean an operator or a 1+ param function without operands)
		else if istart == Some(0) {
			self.res=Err(RpnError::Exec(ExErr::StartOperandMissing));
		}
		else { while let Some(i)=istart {
			if let Some(op)=self.parse_stack.get(i) {
				let rv=(op.refdef.fn_eval)(i, &self.parse_stack, op.nb_param);
				if let Ok(res) = rv {
					self.res=Ok(Some(res));
				}
				else {
					self.res=Err(rv.err().unwrap());
					break;
				}
				if let Some(op)=self.parse_stack.get_mut(i) {
					op.fun_exec_done=true;
					if let Ok(Some(val))=&self.res {
						op.val=val.clone();
					}
				}
			} 
			else {
				self.res=Err(RpnError::Exec(ExErr::TokenNotFound));	
				break;
			}
			// look next operator or function 
			istart = self.parse_stack.iter().position(|i| !i.fun_exec_done && ((i.kind==TokKind::Operator) || 
				(i.kind==TokKind::Function && i.nb_param>0)));
			
		}}
		self.res.clone()
	}
}

impl <'a> fmt::Display for Expression <'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.parsed {
			write!(f, "{}", format!("{}", self.display_parsed()))
		}
		else {
			write!(f, "{}", format!("{}", self.display_tokenized()))			
		}
	}
}

// possible states for the lexical analysis
#[derive(PartialEq, Eq)]
enum Lex {
	Start,
	NumHexInit,
	NumSigBdp,
	NumSigAdp,
	NumExpSign,
	NumExpASign,
	NumExpVal,
	NumHex,
	CString,
	Operator,
	FuncConst,
}

// enm for number analysis
enum NumType{
	Integer,
	Hexa,
	Float,
}

// special characters for the lexical analysis
// double quote
const CHAR_QUOTE:char='\u{0022}'; 
const CHAR_OPAR:char='(';
const CHAR_CPAR:char=')';
// plus and minus, not as operator but signs for a float exponent; +/- in front of a number are managed
// as unary operators
const CHAR_PLUS:char='+'; 
const CHAR_MINUS:char='-';
const CHAR_SEP:char=',';
const CHAR_SP:char=' ';
const CHAR_DECPOINT:char='.';

// characters for an hexadecimal number
const HEXA_CHARS:&'static str="0123456789abcdefABCDEF";
// characters for operators
const OP_CHARS:&'static str="+-/*!^&=|<>~";

// operators defintion array 
const OPERATORS:[&'static FuCoOpDef;19]=[
	&FuCoOpDef{name:"+",  prio:9,  fn_eval:eval_plus,   val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"-",  prio:10, fn_eval:eval_sub,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"*",  prio:12, fn_eval:eval_mul,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"/",  prio:11, fn_eval:eval_div,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"==", prio:5,  fn_eval:eval_eq,     val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"!=", prio:5,  fn_eval:eval_neq,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"<=", prio:5,  fn_eval:eval_infeq,  val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:">=", prio:5,  fn_eval:eval_supeq,  val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"<",  prio:5,  fn_eval:eval_inf,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:">",  prio:5,  fn_eval:eval_sup,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"!",  prio:13, fn_eval:eval_lognot, val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"~",  prio:13, fn_eval:eval_bitnot, val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"&&", prio:2,  fn_eval:eval_and,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"||", prio:2,  fn_eval:eval_or,     val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"^",  prio:3,  fn_eval:eval_bitxor, val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"&",  prio:3,  fn_eval:eval_band,   val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"|",  prio:3,  fn_eval:eval_bor,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:"<<", prio:1,  fn_eval:eval_shl,    val:EVar::IVal(0), params:None},
	&FuCoOpDef{name:">>", prio:1,  fn_eval:eval_shr,    val:EVar::IVal(0), params:None},
];

// functions defintion array 
const FUDEF:[&'static FuCoOpDef;13]=[
	&FuCoOpDef{name:"sin",  fn_eval:eval_sin,  params:Some(1), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"cos",  fn_eval:eval_cos,  params:Some(1), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"tan",  fn_eval:eval_tan,  params:Some(1), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"pow",  fn_eval:eval_pow,  params:Some(2), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"sqrt", fn_eval:eval_sqrt, params:Some(1), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"cbrt", fn_eval:eval_cbrt, params:Some(1), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"exp",  fn_eval:eval_exp,  params:Some(1), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"ln",   fn_eval:eval_ln,   params:Some(1), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"log10",fn_eval:eval_log10,params:Some(1), prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"max",  fn_eval:eval_max,  params:None,    prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"min",  fn_eval:eval_min,  params:None,    prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"avg",  fn_eval:eval_avg,  params:None,    prio:0, val:EVar::IVal(0)},
	&FuCoOpDef{name:"now",  fn_eval:eval_now,  params:Some(0), prio:0, val:EVar::IVal(0)},
	];
	
// constant definition array
const CODEF:[&'static FuCoOpDef;20]=[
	&FuCoOpDef{name:"pi",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(std::f64::consts::PI)},
	&FuCoOpDef{name:"π",    fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(std::f64::consts::PI)},
	// Euler's number
	&FuCoOpDef{name:"e",    fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(std::f64::consts::E)},
	// Golden ratio
	&FuCoOpDef{name:"phi",  fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(1.618_033_988_749_894_848_204_586)},
	&FuCoOpDef{name:"Φ",    fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(1.618_033_988_749_894_848_204_586)},
	// plastic number (nombre radiant), rho
	&FuCoOpDef{name:"rho",  fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(1.324_717_957_244_746_025_960_908)},
	&FuCoOpDef{name:"ρ",    fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(1.324_717_957_244_746_025_960_908)},
	// reference: https://physics.nist.gov/cuu/Constants/index.html
	// speed of light
	&FuCoOpDef{name:"c",    fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(299_792_458.0)},
	// gravitational constant
	&FuCoOpDef{name:"G",    fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(6.674_30E-11)},
	// Planck constant
	&FuCoOpDef{name:"h",    fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(6.626_070_15e-34)},
	// Planck mass
	&FuCoOpDef{name:"pm",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(2.176_434e-8)},
	// Planck time
	&FuCoOpDef{name:"pt",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(5.391_247e-44)},
	// Planck length
	&FuCoOpDef{name:"pl",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(1.616_255e-35)},
	// elementary charge
	&FuCoOpDef{name:"qe",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(1.602_176_634e-19)},
	// electron rest mass
	&FuCoOpDef{name:"me",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(9.109_383_7015e-31 )},
	// proton rest mass
	&FuCoOpDef{name:"mp",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(1.672_621_923_69e-27)},
	// neutron rest mass
	&FuCoOpDef{name:"mn",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(1.674_927_498_04e-27)},
	// Avogadro's number
	&FuCoOpDef{name:"NA",   fn_eval:eval_null, params:None, prio:0, val:EVar::FVal(6.022_140_76e23)},
	&FuCoOpDef{name:"true", fn_eval:eval_null, params:None, prio:0, val:EVar::BVal(true)},
	&FuCoOpDef{name:"false",fn_eval:eval_null, params:None, prio:0, val:EVar::BVal(false)},
	];

fn is_operator(c:char) -> bool {
	OP_CHARS.contains(c)
}
fn is_hexa_char(c:char) -> bool {
	HEXA_CHARS.contains(c)
}
fn is_exponent_char(c:char) -> bool {
	c=='e' || c=='E'
}
fn is_hexa_prefix(c:char) -> bool {
	c=='x' || c=='X'
}

fn lex_error (code:LexErr) -> Result<Option<Token>, RpnError> {
	Err(RpnError::AnaLex(code))		
}

impl <'a> Expression <'a> {
	fn get_token_fuco(& self, c:char, token: &mut Token) -> Result<Option<Token>, RpnError> {
		token.svalue.pop();
		if c == CHAR_SEP || c == CHAR_CPAR || c == CHAR_OPAR || c == CHAR_SP || is_operator(c) {
			if let Some(fu) = FUDEF.iter().find(|&elt| elt.name == token.svalue) {
				token.kind=TokKind::Function;
				token.refdef = fu;
				return Ok(Some(token.clone()));
			}
			else if let Some(co) = CODEF.iter().find(|&elt| elt.name == token.svalue) {
				token.kind=TokKind::Constant;
				token.val=co.val.clone();
				token.refdef = co;
				return Ok(Some(token.clone()));
			}
			else if let Some(context)=self.context {
				if let Some(fu) = context.user_fns.iter().find(|&elt| elt.name == token.svalue) {
					token.kind=TokKind::Function;
					token.refdef = fu;
					return Ok(Some(token.clone()));
				}
				else {
					return lex_error(LexErr::FunctionOrConstNotFound);
				}
			}
			else {
				return lex_error(LexErr::FunctionOrConstNotFound);
			}
		}
		return lex_error(LexErr::FunctionOrConst)
	}

	fn get_token_number(& self, c:char, numtype:NumType, errcode:LexErr, token: &mut Token) -> Result<Option<Token>, RpnError> {
		if is_operator(c) || c == CHAR_CPAR || c == CHAR_SEP || c == CHAR_SP {
			token.svalue.pop();
			token.kind = TokKind::Number;
			match numtype {
				NumType::Integer => {
					if let Ok(i) = token.svalue.parse::<i64>() {
						token.val=EVar::IVal(i);
					}
					else {
						return lex_error(errcode);
					}
				},
				NumType::Hexa => {
					if let Ok(i) = i64::from_str_radix(token.svalue.trim_start_matches("0x"), 16){
						token.val=EVar::IVal(i);
					}
					else {
						return lex_error(errcode);
					}
				},
				NumType::Float => {
					if let Ok(f) = token.svalue.parse::<f64>() {
						token.val=EVar::FVal(f);
					}
					else {
						return lex_error(errcode);
					}
				},
			}
			return Ok(Some(token.clone()));
		}
		lex_error(errcode)
	}

	fn get_token_operator(& self, errcode:LexErr, token: &mut Token) -> Result<Option<Token>, RpnError> {
		if let Some(op) = OPERATORS.iter().find(|&&s| *s.name == token.svalue) {
			token.kind = TokKind::Operator;
			token.refdef=op;
			return Ok(Some(token.clone()));
		} 
		lex_error(errcode)
	}

	fn get_token(& self, sce: &mut Chars) -> Result<Option<Token>, RpnError> {
		let mut cpy = sce.clone();
		let mut token = Token::new();
		let mut step=Lex::Start;
		//while let Some(cc) = cpy.next()  {
		loop {
			let cc = cpy.next();
			// needed to manage the last character in the string to tokenize; when the end
			// of the string is reached, the 'space' character is fed into the fsm one last
			// time (last=true) to force the completion of the analysis of the current token
			let mut c = ' ';
			let mut last=false;
			if cc.is_none() {
				last=true;
			}
			else {
				c=cc.unwrap();
			}
			if c.is_control() || c.is_whitespace() {
				if step == Lex::Start {
					if last {
						break;
					}
					sce.next();
					continue;
				}
				else if step != Lex::CString {
					c=' ';
				}
			}
			token.svalue.push(c);
			if step == Lex::Start {
				if c == '0' { // possible hexa
					step=Lex::NumHexInit;
				}
				else if c.is_numeric() { // number, non hexa
					step=Lex::NumSigBdp;
				}
				else if c == CHAR_DECPOINT { //number, after dec point
					step=Lex::NumSigAdp;
				}
				else if c == CHAR_QUOTE { // string
					step=Lex::CString;
				}
				else if is_operator(c) { // operator
					step=Lex::Operator;
				}
				else if c.is_alphabetic() { // function or constant				
					step=Lex::FuncConst;
				}
				else if (c == CHAR_OPAR) || (c == CHAR_CPAR) { //opening/closing parenthesis
					token.kind = if c == CHAR_OPAR {TokKind::OPar} else {TokKind::CPar};
					sce.next();
					return Ok(Some(token));
				}
				else if c == CHAR_SEP { //separator
					token.kind = TokKind::Separator;
					sce.next();
					return Ok(Some(token));
				}
				else {
					return lex_error(LexErr::BadStart);
				}
			}
			else if step == Lex::NumHexInit {
				if is_hexa_prefix(c) {
					step=Lex::NumHex;
				}
				else if c.is_numeric() {
					step=Lex::NumSigBdp;
				}
				else if c == CHAR_DECPOINT { 
					step=Lex::NumSigAdp;
				}
				else {
					return self.get_token_number(c, NumType::Integer, LexErr::BadHexInit, &mut token);
				}
			}
			else if step == Lex::NumSigBdp {
				if c.is_numeric() {
					step=Lex::NumSigBdp;
				}
				else if c == CHAR_DECPOINT {
					step=Lex::NumSigAdp;
				}
				else if  is_exponent_char(c) {
					step=Lex::NumExpSign;
				}
				else {
					return self.get_token_number(c, NumType::Integer, LexErr::BadSigBdp, &mut token);
				}
			}
			else if step == Lex::NumSigAdp {
				if c.is_numeric() {
					step=Lex::NumSigAdp;
				}
				else if  is_exponent_char(c) {
					step=Lex::NumExpSign;
				}
				else {
					return self.get_token_number(c, NumType::Float, LexErr::BadSigAdp, &mut token);
				}
			}
			else if step == Lex::NumExpSign {
				if c.is_numeric() {
					step=Lex::NumExpVal;
				}
				else if  c == CHAR_PLUS || c == CHAR_MINUS {
					step=Lex::NumExpASign;
				}
				else {
					return lex_error(LexErr::BadExpSign);
				}
			}
			else if step == Lex::NumExpASign {
				if c.is_numeric() {
					step=Lex::NumExpVal;
				}
				else {
					return lex_error(LexErr::BadExpASign);
				}
			}
			else if step == Lex::NumExpVal {
				if c.is_numeric() {
					step=Lex::NumExpVal;
				}
				else {
					return self.get_token_number(c, NumType::Float, LexErr::BadExpVal, &mut token);
				}
			}
			else if step == Lex::NumHex {
				if is_hexa_char(c) {
					step=Lex::NumHex;
				}
				else {
					return self.get_token_number(c, NumType::Hexa, LexErr::BadHex, &mut token);
				}
			}
			else if step == Lex::Operator {
				if is_operator(c) {
					sce.next();
					return self.get_token_operator(LexErr::BadOperatorLong, &mut token);
				}
				else if c == CHAR_QUOTE {
					return lex_error(LexErr::QuoteAfterOp);
				}
				else {
					token.svalue.pop();
					return self.get_token_operator(LexErr::BadOperatorShort, &mut token);
				}
			}
			else if step == Lex::CString {
				if c == CHAR_QUOTE {
					token.kind = TokKind::CString;
					// copy the string without the double quote at the begining and at the end
					// works even for an utf-8 string as the quote character is single byte 
					// NB: len() gives the string's number of bytes
					token.val=EVar::SVal(String::from(&token.svalue[1..token.svalue.len() - 1]));
					sce.next();
					return Ok(Some(token));
				}
			}
			else if step == Lex::FuncConst {
				if c.is_alphabetic() || c.is_numeric() {
					step = Lex::FuncConst;
				}
				else {
					return self.get_token_fuco(c, &mut token);
				}
			}
			sce.next();
			if last {
				break;
			}
		}
		Ok(None)
	}
}
