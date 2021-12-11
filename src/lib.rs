use core::str::Chars;
use std::fmt;
use std::cell::Cell;
mod srlvariant;
use crate::srlvariant::*;

#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub enum ParErr {
	ParamNb,
	MatchingPar,
}
#[derive(Debug, Clone, Copy)]
pub enum ExErr {
	NotParsed,
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

#[derive(Debug, Clone, Copy)]
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

enum NumType{
	Integer,
	Hexa,
	Float,
}

//type RpnEvalFn = fn(istart:usize, tokens:&mut TokenVec) -> Result<Option<TVal>, RpnError>;
type RpnEvalFn = fn(i:usize, tokens:& TokenVec, nb_param:u32) -> Result<EVar, RpnError>;

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Copy, Clone)]
pub enum TokenId {
	Oper(OpId),
	FuCo(FuCoId),
}

#[derive(Clone)]
pub struct Token {
	kind: TokKind,
	id: Option<TokenId>,
	prec: u32,
	unary: bool,
	svalue: String,
	val: EVar,
	fn_eval: Option<RpnEvalFn>,
	params: Option<u32>,
	nb_param: u32,
	val_consumed: Cell<bool>,
	fun_exec_done: bool,
}

impl Token {
	fn new() -> Token {
		Token{kind:TokKind::Void, id:None, svalue:String::from(""),
		val:EVar::IVal(0), fn_eval: None, prec:0, unary:false, params:None, nb_param:0, val_consumed:Cell::new(false), fun_exec_done:false}
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

pub struct TokenVec {
	pub vec:Vec<Token>,
}

impl fmt::Display for TokenVec {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut msg=String::from("");
		for val in self.vec.iter() {
			match val.kind {
				TokKind::Function => msg.push_str(&*format!("{}({}) ", val.svalue, val.nb_param)),
				TokKind::Operator => msg.push_str(&*format!("{}({}) ", val.svalue, 
					if val.unary {"un"} else {"bi"})),
				_ => msg.push_str(&*format!("{} ", val.svalue)),
			}
		}		
		write!(f, "{}", msg)
	}
}

// main component of the rpn crate; contains:
// - a list of tokens and various status
// - methodes to tokenize, parse and evaluate an expression
pub struct Expression {
	parse_stack:TokenVec,
	tokens:TokenVec,
	tokenized:bool,
	parsed:bool,
	err:Option<RpnError>,
	reseval:Option<EVar>,
}

impl Expression {
	pub fn new(exp: &str) -> Expression {
		let mut toks=Expression {
			tokens:TokenVec{vec:vec![]},parse_stack:TokenVec{vec:vec![]}, 
			tokenized:false, parsed:false, err:None, reseval:None};
		let rv=toks.parse(exp);
		if rv.is_ok() {
			println!("expression \'{}\' successully parsed as \'{}\'", exp, toks);
		}
		else {
			println!("expression \'{}\' could not be parsed with error {}", exp, rv.err().unwrap());
			toks.err = rv.err();
		}
		toks
	}
	// fn tokenize generates a list of token from an expression (param 'exp')
	pub fn tokenize(&mut self, exp: &str) -> Result<(), RpnError> {
		let mut exp_iter=exp.chars();
		let mut resok=true;
		let mut error=RpnError::None;
		self.tokens.vec.clear();
		loop {
			let rv=get_token(&mut exp_iter);
			match rv {
				Ok(v) => {
					if v.is_none() {
						break;
					}
					else {
						self.tokens.vec.push(v.unwrap());
					}
				},
				Err(e) => {
					println!("get_token error {:#?}", e);
					resok=false;
					error=e;
					break;
				}
			}		
		}
		if resok {
			self.tokenized=true;
			self.err=None;
			println!("expression \'{}\' successully tokenized as {}", exp, self.tokens);
			return Ok(());
		}
		println!("expression \'{}\' could not be tokenized", exp);
		self.tokenized=false;
		self.err=Some(error.clone());
		Err(error)
	}
	// fn parse creates a rpn stack from the expression passed as parameter
	// the "parsed" rpn stack is then ready for evaluation
	pub fn parse(&mut self, exp: &str) -> Result<(), RpnError>  {
		self.parse_stack.vec.clear();
		let mut op_stack = TokenVec{vec:vec![]};	
		let mut resok=true;
		let mut prev_tok_kind=TokKind::Void;
		let mut error=RpnError::None;
		if !self.tokenized {
			self.tokenize(exp)?;
		}
		for rv in &self.tokens.vec {
			let mut nb_param;
			let mut tok=rv.clone();
			let mut to_stack_direct=false;
			let prev_kind=tok.kind;
			if tok.is_operand() {
				// this is an operand, to be stacked directly in the parse stack
				self.parse_stack.vec.push(tok);
				if let Some(last_op)=op_stack.vec.last() {
					// if in addition the last element of the operators stack is an unary operator, it is
					// moved to the parse stack
					if last_op.is_unary() {
						self.parse_stack.vec.push(last_op.clone());
						op_stack.vec.pop();
					}
				}
			}
			else {
				match tok.kind {
					TokKind::Function | TokKind::OPar | TokKind::Separator => {
						// new token is a function or opening parenthesis, 
						// to stack directly in the operators stack
						op_stack.vec.push(tok);
					},
					TokKind::Operator => {
						// new token is an operator
						// if this is the first, stack it aside
						// else, it depends of what has been put aside before
						// if the previous token put aside is a function or opening parenthesis, or
						// if the previous token is an operator with a lower precedence, the new operator
						// is stacked aside
						if op_stack.vec.len()==0 {
							match prev_tok_kind {
								TokKind::Void => tok.set_unary(true),
								_ => tok.set_unary(false),
							};
							op_stack.vec.push(tok);
						}
						else {
							let last_op=op_stack.vec.last().unwrap();
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
									if last_op.prec>0 && last_op.prec<=tok.prec && !last_op.is_unary() {
										to_stack_direct=true;
									}
								},
							};
							if !to_stack_direct {
								while let Some(last_op)=op_stack.vec.last() {
									if last_op.is_generic_sep() {
										break;
									}
									if (last_op.prec>0 && last_op.prec > tok.prec) || last_op.is_unary(){
										self.parse_stack.vec.push(last_op.clone());
										op_stack.vec.pop();
									}
									else {
										break;
									}
								}
							}
							op_stack.vec.push(tok);									
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
						while let Some(last_op)=op_stack.vec.last() {
							if last_op.is_opar() {
								op_stack.vec.pop();
								match_found=true;
								break;
							}
							else {
								if last_op.is_param_sep() {
									nb_param += 1;
								}
								else {
									// no separator nor parentheses in the parse stack
									self.parse_stack.vec.push(last_op.clone());
								}
								op_stack.vec.pop();
							}
						}
						if !match_found {
							// the matching opening parenthesis was not found, this is an error
							resok=false;
							error=RpnError::Parse(ParErr::MatchingPar);
							break;
						}
						if let Some(last_op)=op_stack.vec.last() {
							if last_op.is_function() {
								let mut op=last_op.clone();
								op.nb_param=nb_param;
								if let Some(params)=op.params {
									if params != nb_param {
										resok=false;
										error=RpnError::Parse(ParErr::ParamNb);
										break;
									}
								}
								self.parse_stack.vec.push(op);
								op_stack.vec.pop();
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
			while let Some(last_op)=op_stack.vec.last() {
				if last_op.is_opar() {
					// should not happen, this is an orphan parenthesis
					resok=false;
					error=RpnError::Parse(ParErr::MatchingPar);
					break;
				}
				self.parse_stack.vec.push(last_op.clone());
				op_stack.vec.pop();
			}
		}
		
		if resok {
			//println!("{}", parse_stack);
			self.parsed=true;
			self.err=None;
			return Ok(());
		}
		println!("parse stack {}", self.parse_stack);
		self.parsed=false;
		self.err=Some(error.clone());
		Err(error)
	}
	pub fn eval(&mut self) -> Result<Option<EVar>, RpnError> {
		if !self.parsed {
			if self.err.is_none() {
				//self.compile
			}
			else {
				return Err(RpnError::Exec(ExErr::NotParsed));
			}
		}
		self.reseval=None;
		self.err=None;
		// reset exec flags of all tokens in the stack
		for val in &mut self.parse_stack.vec {
			val.val_consumed.set(false);
			val.fun_exec_done = false;
		}
		// find the position of the first operator or non-zero param function
		let mut istart = self.parse_stack.vec.iter().position(|i| (i.kind==TokKind::Operator) || 
			(i.kind==TokKind::Function && i.nb_param>0));
		// no operator nor function, meaning the stack must have one element (number, constant or zero-param function)
		// and the result for the evaluation is the value of this single element
		if istart.is_none() {
			if self.parse_stack.vec.len() == 1 {
				if let Some(op)=self.parse_stack.vec.get(0) {
					match op.kind {
						TokKind::Number | /*TokKind::Operator |*/ TokKind::Constant => self.reseval=Some(op.val),
						TokKind::Function => {
							if op.nb_param>0 {
								self.err=Some(RpnError::Exec(ExErr::TooManyParams))
							}
							else {
								let rv=op.fn_eval.unwrap()(0, & self.parse_stack, 0);
								if rv.is_ok() {
									self.reseval=Some(rv.ok().unwrap());
								}
								else {
									self.err=rv.err();
								}
							}
						}
						_ => self.err=Some(RpnError::Exec(ExErr::StartWrongToken)),
					}
				}
			}
			else {
				self.err=Some(RpnError::Exec(ExErr::WrongStackLen));			
			}
		}
		// impossible (that would mean an operator or a 1+ param function without operands)
		else if istart == Some(0) {
			self.err=Some(RpnError::Exec(ExErr::StartOperandMissing));
		}
		else { while let Some(i)=istart {
			//println!("eval i={}", i);
			if let Some(op)=self.parse_stack.vec.get(i) {
				if let Some(fu)=op.fn_eval {
					let rv=fu(i, &self.parse_stack, op.nb_param);
					if rv.is_ok() {
						self.reseval=Some(rv.ok().unwrap());
					}
					else {
						self.err=rv.err();
						break;
					}
				}
				if let Some(op)=self.parse_stack.vec.get_mut(i) {
					op.fun_exec_done=true;
					op.val=self.reseval.unwrap();
				}
			} 
			else {
				self.err=Some(RpnError::Exec(ExErr::TokenNotFound));	
				break;
			}
			// look next operator or function 
			istart = self.parse_stack.vec.iter().position(|i| !i.fun_exec_done && ((i.kind==TokKind::Operator) || 
				(i.kind==TokKind::Function && i.nb_param>0)));
			
		}}
		if let Some(failed)=self.err {
			return Err(failed);
		}
		Ok(self.reseval)
	}
}

impl fmt::Display for Expression {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", format!("{}", self.parse_stack))
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

#[derive(Debug, Copy, Clone)]
pub enum OpId {
	Plus,
	Minus,
	Mul,
	Div,
	Equal,
}

struct Operator {
	id:TokenId,
	prec: u32,
	name:&'static str,
	fn_eval:RpnEvalFn,
}

const CHAR_QUOTE:char='\u{0022}';
const CHAR_OPAR:char='(';
const CHAR_CPAR:char=')';
const CHAR_PLUS:char='+';
const CHAR_MINUS:char='-';
const CHAR_SEP:char=',';
const CHAR_SP:char=' ';
const CHAR_DECPOINT:char='.';

const HEXA_CHARS:&'static str="0123456789abcdefABCDEF";
const OP_CHARS:&'static str="+-/*!^&=|<>";
const OPERATORS:[&'static Operator;5]=[
	&Operator{id:TokenId::Oper(OpId::Plus), prec:1, name:"+", fn_eval:eval_plus},
	&Operator{id:TokenId::Oper(OpId::Minus), prec:1, name:"-", fn_eval:eval_sub},
	&Operator{id:TokenId::Oper(OpId::Mul), prec:10, name:"*", fn_eval:eval_mul},
	&Operator{id:TokenId::Oper(OpId::Div), prec:11, name:"/", fn_eval:eval_div},
	&Operator{id:TokenId::Oper(OpId::Equal), prec:1, name:"==", fn_eval:eval_eq}
];

#[derive(Debug, Copy, Clone)]
// common id functions/constants
pub enum FuCoId {
	Sin,
	Cos,
	Pow,
	Max,
	Avg,
	Pi,
	Euler,
	True,
	False,
}
struct FuDef {
	id: TokenId,
	params: Option<u32>,
	name:&'static str,
	fn_eval:RpnEvalFn,
}	
struct CoDef {
	id: TokenId,
	name:&'static str,
	val:EVar,
}	

const FUDEF:[&'static FuDef;5]=[
	&FuDef{fn_eval:eval_sin, id:TokenId::FuCo(FuCoId::Sin), params:Some(1), name:"sin"},
	&FuDef{fn_eval:eval_cos, id:TokenId::FuCo(FuCoId::Cos), params:Some(1), name:"cos"},
	&FuDef{fn_eval:eval_pow, id:TokenId::FuCo(FuCoId::Pow), params:Some(2), name:"power"},
	&FuDef{fn_eval:eval_max, id:TokenId::FuCo(FuCoId::Max), params:None, name:"max"},
	&FuDef{fn_eval:eval_avg, id:TokenId::FuCo(FuCoId::Avg), params:None, name:"avg"},
	];
const CODEF:[&'static CoDef;5]=[
	&CoDef{id:TokenId::FuCo(FuCoId::Pi), name:"pi", val:EVar::FVal(std::f64::consts::PI)},
	&CoDef{id:TokenId::FuCo(FuCoId::Pi), name:"π", val:EVar::FVal(std::f64::consts::PI)},
	&CoDef{id:TokenId::FuCo(FuCoId::Euler), name:"e", val:EVar::FVal(std::f64::consts::E)},
	&CoDef{id:TokenId::FuCo(FuCoId::True), name:"true", val:EVar::BVal(true)},
	&CoDef{id:TokenId::FuCo(FuCoId::False), name:"false", val:EVar::BVal(false)},
	];

/* 
version with lazy_static and a HashMap for functions/constants
works just as well without lazy_static and a plain static array of struct (cf above)
#[derive(Debug)]
enum RpnFunctionId {
	Sin,
	Cos,
}
struct RpnFunction {
	id: RpnFunctionId,
	params: Option<i32>,
}
#[derive(Debug)]
enum RpnConstantId {
	Pi,
	Euler,
}
struct RpnConstant {
	id: RpnConstantId,
}

lazy_static! {
    static ref FUNCTIONSMAP: HashMap<&'static str, &'static RpnFunction> = {
        let mut m = HashMap::new();
        m.insert("sin", &{RpnFunction {id:RpnFunctionId::Sin, params:Some(1)}});
        m.insert("cos", &{RpnFunction {id:RpnFunctionId::Cos, params:Some(1)}});
        m.insert("test", &{RpnFunction {id:RpnFunctionId::Cos, params:None}});
        m
    };
    static ref FUNCTIONSCOUNT: usize = FUNCTIONSMAP.len();
    static ref CONSTANTSMAP: HashMap<&'static str, &'static RpnConstant> = {
        let mut m = HashMap::new();
        m.insert("pi", &{RpnConstant {id:RpnConstantId::Pi}});
        m.insert("e", &{RpnConstant {id:RpnConstantId::Euler}});
        m
    };
    static ref CONSTANTSCOUNT: usize = CONSTANTSMAP.len();
}
*/
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
	//Err(RpnError {code: code,})		
	Err(RpnError::AnaLex(code))		
}
/*
fn lex_error_ex (code:usize) -> Result<(), RpnError> {
	Err(RpnError {code: code,})		
}
*/
// two possible versions
// 1. returns a Result<Option<Token>, RpnError>
// 2. returns a Result<(), RpnError>
// pb with version 1 is that you need to clone the mutable token reference passed as parameters
// which is somehow wasteful as the right token is already modified. Maybe it is optimized
// by the rust comiler?
fn get_token_fuco(c:char, token: &mut Token) -> Result<Option<Token>, RpnError> {
	token.svalue.pop();
	if c == CHAR_SEP || c == CHAR_CPAR || c == CHAR_OPAR || c == CHAR_SP || is_operator(c) {
		if let Some(fu) = FUDEF.iter().find(|&&s| *s.name == token.svalue) {
			token.kind=TokKind::Function;
			token.fn_eval=Some(fu.fn_eval);
			token.id = Some(fu.id);
			token.params = fu.params;
			return Ok(Some(token.clone()));//rather than Ok(());
		}
		else if let Some(co) = CODEF.iter().find(|&&s| *s.name == token.svalue) {
			token.kind=TokKind::Constant;
			token.val=co.val;
			token.id = Some(co.id);
			return Ok(Some(token.clone()));//rather than Ok(());
		}
		else {
			return lex_error(LexErr::FunctionOrConstNotFound);
		}
	}
	return lex_error(LexErr::FunctionOrConst)
}

fn get_token_number(c:char, numtype:NumType, errcode:LexErr, token: &mut Token) -> Result<Option<Token>, RpnError> {
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

fn get_token_operator(errcode:LexErr, token: &mut Token) -> Result<Option<Token>, RpnError> {
	if let Some(op) = OPERATORS.iter().find(|&&s| *s.name == token.svalue) {
		token.kind = TokKind::Operator;
		token.id = Some(op.id);
		token.prec = op.prec;
		token.fn_eval = Some(op.fn_eval);
		return Ok(Some(token.clone()));
	} 
	lex_error(errcode)
}

fn get_token(sce: &mut Chars) -> Result<Option<Token>, RpnError> {
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
				return get_token_number(c, NumType::Integer, LexErr::BadHexInit, &mut token);
				//return if let Err(e)=get_token_number(c, NumType::Integer, 2, &mut token) 
				//	{Err(e)} else {Ok(Some(token))};
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
				return get_token_number(c, NumType::Integer, LexErr::BadSigBdp, &mut token);
				//return if let Err(e)=get_token_number(c, NumType::Integer, 3, &mut token) 
				//	{Err(e)} else {Ok(Some(token))};
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
				return get_token_number(c, NumType::Float, LexErr::BadSigAdp, &mut token);
				//return if let Err(e)=get_token_number(c, NumType::Float, 4, &mut token) 
				//	{Err(e)} else {Ok(Some(token))};
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
				return get_token_number(c, NumType::Float, LexErr::BadExpVal, &mut token);
				//return if let Err(e)=get_token_number(c, NumType::Float, 7, &mut token) 
				//	{Err(e)} else {Ok(Some(token))};
			}
		}
		else if step == Lex::NumHex {
			if is_hexa_char(c) {
				step=Lex::NumHex;
			}
			else {
				return get_token_number(c, NumType::Hexa, LexErr::BadHex, &mut token);
				//return if let Err(e)=get_token_number(c, NumType::Hexa, 8, &mut token) 
				//	{Err(e)} else {Ok(Some(token))};
			}
		}
		else if step == Lex::Operator {
			if is_operator(c) {
				sce.next();
				return get_token_operator(LexErr::BadOperatorLong, &mut token);
				//return if let Err(e)=get_token_operator(9, &mut token) {Err(e)} else {Ok(Some(token))};
			}
			else if c == CHAR_QUOTE {
				return lex_error(LexErr::QuoteAfterOp);
			}
			else {
				token.svalue.pop();
				return get_token_operator(LexErr::BadOperatorShort, &mut token);
				//return if let Err(e)=get_token_operator(11, &mut token) {Err(e)} else {Ok(Some(token))};
			}
		}
		else if step == Lex::CString {
			if c == CHAR_QUOTE {
				token.kind = TokKind::CString;
				sce.next();
				return Ok(Some(token));
			}
		}
		else if step == Lex::FuncConst {
			if c.is_alphabetic() {
				step = Lex::FuncConst;
			}
			else {
				return get_token_fuco(c, &mut token);
				//return if let Err(e)=get_token_fuco(c, &mut token) {Err(e)} else {Ok(Some(token))};
				/* version with lazy_static and a HashMap
				token.svalue.pop();
				if c == CHAR_SEP || c == CHAR_CPAR || is_operator(c) {
					if let Some(co) = CONSTANTSMAP.get(&*token.svalue) {
						token.kind=TokKind::Constant;
						println!("constant id {:#?}", co.id);
						return Ok(Some(token));
					}
					else {
						return lex_error(12);
					}
				}
				else if c == CHAR_OPAR {
					if let Some(fu) = FUNCTIONSMAP.get(&*token.svalue) {
						token.kind=TokKind::Function;
						println!("constant id {:#?}", fu.id);
						return Ok(Some(token));
					}
					else {
						return lex_error(13);
					}
				}
				else {
					return lex_error(14);
				}
				*/
			}
		}
		sce.next();
		if last {
			break;
		}
	}
	Ok(None)
}

// look for nb operand(s) in tokens from position i, going backwards
// returns a tuple: index of the token found in tokens and a reference to this token
// NB side effects: the token found will be consumed (val_consumed set to true)
fn get_operand (istart:usize, tokens:& TokenVec) ->  Result<(usize, &Token), RpnError> {
	if istart==0 {
		return Err(RpnError::Exec(ExErr::GetOperandStart));
	}
	for i in (0..istart).rev() {
		if let Some(op)=tokens.vec.get(i) {
			if op.val_consumed.get()==false /*&& 
				(op.kind==TokKind::Number || op.kind==TokKind::Constant || op.kind==TokKind::CString)*/ {
				op.val_consumed.set(true);
				return Ok((i,op));
			}
		}
	}
 	Err(RpnError::Exec(ExErr::GetOperandMissing))
}

fn eval_plus(i:usize, tokens:& TokenVec, nb_param:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	if nb_param == 1 { // nb_param=1 => unary operator
		return Ok(op1.val);
	}
	else {
		let (_, op2) = get_operand (i_1, tokens)?;
		return Ok(op2.val+op1.val);
	}
}

fn eval_sub(i:usize, tokens:& TokenVec, nb_param:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	if nb_param == 1 { // nb_param=1 => unary operator
		return Ok(EVar::IVal(0)-op1.val);
	}
	else {
		let (_, op2) = get_operand (i_1, tokens)?;
		return Ok(op2.val-op1.val);
	}
}

fn eval_mul(i:usize, tokens:& TokenVec, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(op2.val*op1.val);
}

fn eval_div(i:usize, tokens:& TokenVec, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	match op1.val {
			EVar::IVal(i) => if i == 0 {return Err(RpnError::Exec(ExErr::EvalDiv0));}
			EVar::FVal(f) => if f == 0.0 {return Err(RpnError::Exec(ExErr::EvalDiv0));}
			EVar::BVal(b) => if b == false {return Err(RpnError::Exec(ExErr::EvalDiv0));}
	}
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(op2.val/op1.val)
}

fn eval_eq(i:usize, tokens:& TokenVec, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(EVar::BVal(op2.val == op1.val));
}

fn eval_sin(i:usize, tokens:& TokenVec, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.sin());
}

fn eval_cos(i:usize, tokens:& TokenVec, _:u32) -> Result<EVar, RpnError> {
	let (_, op1) = get_operand (i, tokens)?;
	return Ok(op1.val.cos());
}

fn eval_pow(i:usize, tokens:& TokenVec, _:u32) -> Result<EVar, RpnError> {
	let (i_1, op1) = get_operand (i, tokens)?;
	let (_, op2) = get_operand (i_1, tokens)?;
	return Ok(op2.val.pow(&op1.val));
}

fn eval_max(i:usize, tokens:& TokenVec, nb_param:u32) -> Result<EVar, RpnError> {
	if nb_param>0 {
		let (j, opj) = get_operand (i, tokens)?;
		let mut idx=j;
		let mut rv=opj.val;
		for _ in 0..nb_param -1 {
			let (k, opk) = get_operand (idx, tokens)?;
			rv=rv.max(opk.val);
			idx=k;
		}
		return Ok(rv);
	}
	Err(RpnError::Exec(ExErr::EvalMaxParam))
}

fn eval_avg(i:usize, tokens:& TokenVec, nb_param:u32) -> Result<EVar, RpnError> {
	if nb_param>0 {
		let (j, opj) = get_operand (i, tokens)?;
		let mut idx=j;
		let mut rv=opj.val;
		for _ in 0..nb_param -1 {
			let (k, opk) = get_operand (idx, tokens)?;
			rv = rv + opk.val;
			idx=k;
		}
		return Ok(rv/EVar::FVal(nb_param as f64));
	}
	Err(RpnError::Exec(ExErr::EvalAvgParam))
}
