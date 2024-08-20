// Copyright (c) 2024-2026 Jonathan A. Thomason

use std::io;
use std::io::Write;
use std::thread;
use std::time;

// This function does not check alignment
pub fn try_print(buf: &[u8]) -> usize {
	let mut consumed_bytes = 0;
	for chunk in buf.utf8_chunks() {
		for ch in chunk.valid().chars() {
			let ch_len = ch.len_utf8();
			for _ in 0..ch_len {
				thread::sleep(time::Duration::from_micros(100));
			}
			consumed_bytes += ch_len;
			if ch == '\0' {
				return consumed_bytes;
			}
			print!("{ch}");
			io::stdout().flush().unwrap();
		}
		for byte in chunk.invalid() {
			thread::sleep(time::Duration::from_micros(100));
			consumed_bytes += 1;
			print!("\\x{:02X}", byte);
			io::stdout().flush().unwrap();
		}
	}
	consumed_bytes
}

pub fn read_line() -> io::Result<String> {
	let mut buf = String::new();
	io::stdin().read_line(&mut buf).map(|_| buf)
}
