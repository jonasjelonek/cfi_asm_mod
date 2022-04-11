use std::fs::File;
use std::path::PathBuf;
use std::io::{BufWriter, Read, Write};
use std::env;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

macro_rules! set_bit {
	($val:expr, $bit:expr) => {
		$val = $val | (0x1_u16 << ($bit as u16))
	};
}

#[allow(unused)]
fn parse_reglist(line: String) -> u16 {
	let regs: Vec<(&str, &str)> = vec![("r3", "NOP"), ("r4", "NOP"), ("r5", "NOP"), ("r6", "NOP"), ("r7", "NOP"), ("r8", "NOP"), ("r9", "NOP"), ("r10", "sl"), ("r11", "fp") ];
	let mut reg_mask: u16 = 0;
	for i in 0..regs.len() {
		if line.contains(regs[i].0) || line.contains(regs[i].1) {
			set_bit!(reg_mask, i);
		}
	}
	println!("Parsed reglist of '{}': {}", line, reg_mask);
	reg_mask
}

fn process_target(path: PathBuf) {
	//println!("File: '{}'", path.to_str().unwrap());
	let mut file: File = match File::open(&path) {
		Ok(x) => x,
		Err(e) => panic!("[{}]: Cannot find or access file '{}': {}", APP_NAME, path.to_str().unwrap(), e) 
	};

	let mut str_content: String = String::new();
	match file.read_to_string(&mut str_content) {
		Ok(_) => {},
		Err(e) => panic!("[{}]: Could not read content of file '{}': {}", APP_NAME, path.to_str().unwrap(), e)
	};
	let asm_lines: Vec<&str> = str_content.lines().collect::<Vec<&str>>();
	let mut asm_lines_mut: Vec<String> = asm_lines.iter().map(|&a| String::from(a)).collect();

	let mut line_counter: u32 = 1;
	for i in 0..asm_lines_mut.len() {
		if asm_lines_mut[i].contains("pop") && asm_lines_mut[i].contains("pc") {
			if asm_lines_mut[i].contains("r8") 
				|| asm_lines_mut[i].contains("r9") 
				|| asm_lines_mut[i].contains("r10") 
				|| asm_lines_mut[i].contains("r11")
				|| asm_lines_mut[i].contains("r12") 
				|| asm_lines_mut[i].contains("fp") 
				|| asm_lines_mut[i].contains("sl") 
				|| asm_lines_mut[i].contains("ip")
			{
				println!("[{}]: Found POP T2 in line {}", APP_NAME, line_counter);

				let jump: &str = "	b	_cfi_check_ra\n";
				asm_lines_mut[i] = asm_lines_mut[i].replace("pc", "lr");
				asm_lines_mut.insert(i + 1, jump.to_string());
			} else {
				println!("[{}]: Found POP T1/T3 in line {}", APP_NAME, line_counter);

				let jump: &str = "	b	_cfi_check_ra\n";
				asm_lines_mut[i] = asm_lines_mut[i].replace("pc", "lr");
				asm_lines_mut.insert(i + 1, jump.to_string());
			}
		} else if asm_lines_mut[i].contains("ldr") && asm_lines_mut[i].contains("pc") && asm_lines_mut[i].contains("[sp]") {
			println!("[{}]: Found POP T3 in line {}", APP_NAME, line_counter);
			let jump: &str = "	b	_cfi_check_ra\n";
			asm_lines_mut[i] = String::from("pop	{lr}");
			asm_lines_mut.insert(i + 1, jump.to_string());
		} else if asm_lines_mut[i].contains("bx") && asm_lines_mut[i].contains("lr") {
			println!("[{}]: Found BX LR in line {}", APP_NAME, line_counter);
			let jump: &str = "	b	_cfi_check_ra\n";
			asm_lines_mut[i] = String::from(jump);
		}
		line_counter += 1;
	}
	asm_lines_mut.insert(0, "	.extern	_cfi_check_ra".to_string());

	let mut bytestream: Vec<u8> = Vec::new();
	for line in asm_lines_mut.iter() {
		bytestream.extend(line.as_bytes());
		bytestream.push('\n' as u8);
	}

	let mod_file: File = File::create(&path).unwrap();
	let mut writer: BufWriter<File> = BufWriter::new(mod_file);
	writer.write_all(&bytestream).unwrap();
}

fn main() {
	let args: Vec<String> = env::args().collect::<Vec<String>>();
	/* args[1] should be parameter, args[2..] further arguments */

	match &(args[1])[..] {
		"-p" => {
			let expand_path = shellexpand::tilde(&args[2]).to_string();
			let path: PathBuf = match PathBuf::from(expand_path).canonicalize() {
				Ok(p) => p,
				Err(e) => {
					println!("[{}]: Could not validate the specified path: {}", APP_NAME, e);
					return;
				},
			};
			process_target(path);
		},
		"-v" => println!("cfi_asm_mod version {}", env!("CARGO_PKG_VERSION")),
		param => {
			println!("[{}]: The parameter {} is not recognized!", APP_NAME, param);
		}
	}
}