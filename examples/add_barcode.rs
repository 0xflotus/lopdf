extern crate lopdf;
use lopdf::xobject;
use lopdf::Document;
use std::str::FromStr;

fn convert_number_to_bits<T: std::fmt::Binary>(num: T, size: usize) -> Vec<u8> {
	let bin = format!("{:b}", num);
	let pad = "0".repeat(size - bin.len());
	let mut bytes = (pad + &bin).into_bytes();
	bytes.reverse();
	bytes
}

fn generate_barcode(page: u32, code: u16) -> Vec<(f64, f64, f64, f64, u8)> {
	assert!(page > 0 && page <= 255, "页码超出了取值范围1-255");
	assert!(code <= 511, "代码超出了取值范围0-511");
	let page_bits = convert_number_to_bits(page, 8);
	let code_bits = convert_number_to_bits(code, 9);
	let mut rects = vec![];
	let mut x = 0.0;
	let y = 0.0;
	let w = 9.0;
	let h = 10.0;
	{
		let mut add_flag = |w, bit| {
			rects.push((x, y, w, h, bit));
			x += w;
		};
		add_flag(w, b'0');
		for bit in page_bits {
			add_flag(w, bit);
		}
		add_flag(w, code_bits[0]);
		add_flag(6.53, b'0');
		add_flag(w, code_bits[1]);
		add_flag(w, code_bits[2]);
		add_flag(w, code_bits[3]);
		add_flag(w, code_bits[4]);
		add_flag(w, b'1');
		add_flag(w, b'0');
		add_flag(w, code_bits[5]);
		add_flag(w, code_bits[6]);
		add_flag(w, code_bits[7]);
		add_flag(w, code_bits[8]);
	}
	return rects;
}

fn generate_operations(rects: Vec<(f64, f64, f64, f64, u8)>) -> String {
	let mut operations = String::new();
	let mut current_color = b'\0';
	for (x, y, w, h, bit) in rects {
		if bit != current_color {
			operations.push_str(match bit {
				b'0' => "1 1 1 rg\n",
				b'1' => "0 0 0 rg\n",
				_ => "\n",
			});
			current_color = bit;
		}
		operations.push_str(&format!("{} {} {} {} re\nf\n", x, y, w, h));
	}
	operations
}

#[allow(non_upper_case_globals)]
const mm2pt: f64 = 2.834;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	assert!(args.len() == 4, "缺少命令行参数");
	let pdf_file = &args[1];
	let code = u16::from_str(&args[2]).expect("error in parsing code argument");
	let output_file = &args[3];
	let mut doc = Document::load(pdf_file).unwrap();
	for (page_number, page_id) in doc.get_pages() {
		let operations = generate_operations(generate_barcode(page_number, code));
		let barcode = xobject::form(
			vec![0.0, 0.0, 595.0 - 12.44 * mm2pt * 2.0, 10.0 * mm2pt],
			vec![mm2pt, 0.0, 0.0, mm2pt, 12.44 * mm2pt, 842.0 - 14.53 * mm2pt],
			operations.as_bytes().to_vec(),
		);
		doc.insert_form_object(page_id, barcode);
	}
	doc.save(output_file).unwrap();
}
