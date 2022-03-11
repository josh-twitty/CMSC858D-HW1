use rand::Rng;
use fast_math::log2;
use std::collections::HashMap;
use std::cmp::min;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct bitvector {
	vector: Vec<i64>,
}

fn naive_rank(v: &Vec<i64>, n: usize, zero: bool) -> i64  {
if n >= v.len() {
	return -1;
}
let mut count = 0; 
for i in 0..(match zero {true => n+1, false => n,}) {
if v[i] == 1 {
count = count + 1;
}
}
count
}

fn test(trials: i64, max_n: i64) {
	let mut i = 4;
	while i < max_n {
		println!("{}", i);
		for j in 0..trials {
		
			let b = bitvector {
				vector: random_vec(i),
				};
			let (v1,v2,v3) = rank_support_structure(&b.vector);
			let supp = rank_support {
				bv: b,
				dataStruct: (v1,v2,v3),
				};
			let r = rand::thread_rng().gen_range(0..i);
			for v in &supp.bv.vector {
			print!("{} ", v);
			}
			print!("{} \n", r);
			
			assert_eq!(supp.rank1(r as u64),naive_rank(&supp.bv.vector, r as usize, true) as u64);
		}
	i = i + 1;
	}

}

fn rank_support_structure(v: &Vec<i64>) -> (Vec<i64>, Vec<i64>, HashMap<(Vec<i64>,i64),i64>) {
let mut v1 = Vec::new();
let mut v2 = Vec::new();
let mut v3 = HashMap::new();
 

let s2 = fast_math::log2_raw(v.len() as f32) as usize;
let s1 = (s2 as f32).powf(2.0) as usize;
v2 = vec![0; (v.len() as f64/ s2 as f64).ceil() as usize];

for i in 0..(v.len() as f64/s1 as f64).ceil() as usize{
let mut count = 0;
for j in 0..i*s1 {
if v[j] == 1 {
	count = count + 1;
}
}
v1.push(count);
}

for k in 0..(v.len() as f64/s2 as f64).ceil() as usize{
	
	v2[k] = naive_rank(v, k*s2, false) - naive_rank(v,(k * s2)/s1 * s1, false);
}

for l in 0..2_u32.pow(s2 as u32) {
for k in 0..s2 {
	v3.insert((int_to_bv(l as i64,s2 as i64),k as i64),naive_rank(&int_to_bv(l as i64,s2 as i64),k,true));
}
if v2.len() * s2 > v.len() {
	let last_block_len = v.len() - (v2.len() - 1)*s2;
	let last_slice = &v[(v2.len()-1)*s2..(v2.len()-1)*s2 + last_block_len];
	for i in 0..last_block_len {
		v3.insert((last_slice.to_vec(), i as i64), naive_rank(&last_slice.to_vec(), i, true));
	}
}
}
return (v1,v2,v3);
}

struct sparse_array {
	ss: Option<select_support>,
	words: Vec<String>,
}

impl sparse_array {
	fn create(&mut self,size: u64) {
		let vector = bitvector{
		vector: vec![0; size.try_into().unwrap()],
		};
		let (v1,v2,v3) = rank_support_structure(&vector.vector);
		let rs = rank_support{
			bv: vector,
			dataStruct: (v1,v2,v3),
		};
		self.ss = Some(select_support{
			rs: rs,
		});
	}
	fn append(&mut self,elem: String,pos: u64) {
		match &mut self.ss {
		Some(supp) => {supp.rs.bv.vector[pos as usize] = 1;
		supp.rs.dataStruct = rank_support_structure(&supp.rs.bv.vector);
		}, 
		None => return,
		}
		self.words.push(elem);
	}
	fn get_at_rank(&self, r: u64, elem: &mut String) -> bool{
		if r >= self.words.len() as u64{
			return false;
		}
		else{
		elem.clear();
		elem.push_str(&(self.words[r as usize]));
		return true;
		}
	}
	
	fn get_at_index(&mut self, r: u64, elem: &mut String) -> bool{
		match &mut self.ss {
		Some(supp) => if r < supp.rs.bv.vector.len().try_into().unwrap() && supp.rs.bv.vector[r as usize] == 1 {
			elem.clear();
			elem.push_str(&(self.words[(supp.rs.rank1(r) - 1) as usize]));
			return true;
		},
		None => return false,
		}
		return false;
	}
	
	fn num_elem_at(&mut self, r: u64) -> u64 {
		match &mut self.ss {
		Some(supp) => if r < supp.rs.bv.vector.len().try_into().unwrap(){
			return supp.rs.rank1(r);
		},
		None => return 0,
		}
		return 0;
	}
	
	fn size(&mut self) -> u64 {
		match &mut self.ss {
		Some(supp) => return supp.rs.bv.vector.len() as u64,
		None => return 0,
		}
	}
	
	fn num_elem(&self) -> u64 {
		return self.words.len() as u64
	}
}

struct rank_support {
	bv: bitvector,
	dataStruct: (Vec<i64>, Vec<i64>, HashMap<(Vec<i64>,i64),i64>),
}

impl rank_support {
	fn rank1(&self, i: u64) -> u64 {
	let n = self.bv.vector.len();
	
	let s2 = fast_math::log2_raw(self.bv.vector.len() as f32) as usize;
	let s1 = (s2 as f32).powf(2.0) as usize;
	let subBlockStart = i as usize - (i as usize%s2);
	
	return (self.dataStruct.0[i as usize/ s1] + self.dataStruct.1[i as usize / s2] + match self.dataStruct.2.get(&(self.bv.vector[subBlockStart..std::cmp::min(self.bv.vector.len(),subBlockStart+s2)].to_vec(), (i % s2 as u64) as i64 )) {
	Some(n) => *n,
	None => -100,
	}) as u64;
	}
	
	fn overhead(&self) -> u64 {
	/* I'm cheating a bit here and working off what the theoretical sizes of things
	should be here rather than what they are. I should have implemented these as 
	with actual bitvectors, but did not.*/
	let n = self.bv.vector.len() as u64;
	return self.bv.vector.len() as u64 + (fast_math::log2(n as f32).ceil() as u64)*self.dataStruct.0.len() as u64 + 
	(fast_math::log2(fast_math::log2(n as f32)).ceil() as u64)*self.dataStruct.1.len() as u64;
	}
	
}


struct select_support {
	rs: rank_support,
}

impl select_support {
	fn select1(&self, i: u64) -> u64 {
		let v = &self.rs.bv.vector;
		let mut left = 0;
		let mut right = (v.len()-1) as u64;
		while left <= right {
			let mut guess = ((left + right)/2) as u64;
			if self.rs.rank1(guess) < i {
				left = guess + 1;
			}
			else if self.rs.rank1(guess) > i {
				right = guess - 1;
			}
			else {
				return guess;
			}
		}
		return v.len() as u64;
	}
}
fn random_vec(n: i64) -> Vec<i64>{
	let mut v = Vec::new();
	for _i in 0..n {
	v.push(rand::thread_rng().gen_range(0..2));
	}
	v
}



fn int_to_bv(n: i64, len: i64) -> Vec<i64>{
let mut quotient = n;
let mut v = Vec::new();
let mut digits = len;
while digits > 0 {
	if quotient != 0 {
		let rem = quotient % 2;
		v.insert(0,rem);
		quotient = quotient / 2;
	}
	else {
		v.insert(0,0)
	}
	digits = digits - 1;
}
v
}


fn main() {
	test(100,100);

	}

