use std::error::Error;
use std::fmt;

const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PADDING: u8 = b'=';

fn decode_char(val: u8) -> Option<usize>
{
	match val
	{
		b'A'..=b'Z' => Some((val - b'A') as usize),
		b'a'..=b'z' => Some(26 + (val - b'a') as usize),
		b'0'..=b'9' => Some(52 + (val - b'0') as usize),
		b'+' => Some(0x3E),
		b'/' => Some(0x3F),
		_ => None
	}
}

pub fn encode(input: &[u8], output: &mut [u8]) -> Result<usize, EncodeError>
{
	let use_pad = input.len() % 3 != 0;
	let expect_len = if use_pad {4 * (input.len() / 3 + 1)} else {4 * (input.len() / 3)};
	if output.len() < expect_len
	{
		return Err(EncodeError::Overflow{need: expect_len, have: output.len()});
	}
	let mut in_pos = 0usize;
	let mut out_pos = 0usize;
	while input.len() - in_pos >= 3
	{
		let buff = ((input[in_pos] as usize) << 16) | ((input[in_pos + 1] as usize) << 8) | (input[in_pos + 2] as usize);
		output[out_pos] = CHARS[buff >> 18];
		output[out_pos + 1] = CHARS[(buff >> 12) & 0x3F];
		output[out_pos + 2] = CHARS[(buff >> 6) & 0x3F];
		output[out_pos + 3] = CHARS[buff & 0x3F];
		out_pos += 4;
		in_pos += 3;
	}
	let remain = input.len() - in_pos;
	if remain > 0
	{
		let mut buff = (input[in_pos] as usize) << 16;
		if remain > 1
		{
			buff |= (input[in_pos + 1] as usize) << 8;
		}
		in_pos += if remain > 1 {2} else {1};
		output[out_pos] = CHARS[buff >> 18];
		output[out_pos + 1] = CHARS[(buff >> 12) & 0x3F];
		if remain > 1
		{
			output[out_pos + 2] = CHARS[(buff >> 6) & 0x3F];
		}
		else {output[out_pos + 2] = PADDING;}
		output[out_pos + 3] = PADDING;
		out_pos += 4;
	}
	assert_eq!(in_pos, input.len(), "missed input ({in_pos}, expected {})", input.len());
	assert_eq!(out_pos, expect_len, "missed output ({out_pos}, expected {expect_len})");
	Ok(out_pos)
}

macro_rules!do_decode
{
	($input:ident, $in_pos:expr) =>
	{
		match decode_char($input[$in_pos])
		{
			None => return Err(DecodeError::Malformed{at: $in_pos, value: $input[$in_pos]}),
			Some(v) => v,
		}
	};
}

pub fn decode(input: &[u8], output: &mut [u8]) -> Result<usize, DecodeError>
{
	if input.len() % 4 != 0
	{
		// can't decode, but check for malformed data first
		let mut in_pad = false;
		for (i, &c) in input.iter().enumerate()
		{
			if c == PADDING
			{
				if i % 4 < 2
				{
					return Err(DecodeError::Malformed{at: i, value: c});
				}
				in_pad = true;
			}
			else if in_pad && i % 4 == 0
			{
				return Err(DecodeError::TrailingData{at: i});
			}
			else if (in_pad && i % 4 == 3) || decode_char(c).is_none()
			{
				return Err(DecodeError::Malformed{at: i, value: c});
			}
		}
		return Err(DecodeError::Truncated);
	}
	let pad_len = if input.len() > 0
	{
		if input[input.len() - 1] != PADDING {0}
		else if input[input.len() - 2] != PADDING {1}
		else {2}
	}
	else {0};
	let expect_len = input.len() / 4 * 3 - pad_len;
	if output.len() < expect_len
	{
		return Err(DecodeError::Overflow{need: expect_len, have: output.len()});
	}
	if !input.is_empty()
	{
		let mut in_pos = 0usize;
		let mut out_pos = 0usize;
		while in_pos < input.len() - 4
		{
			let c0 = do_decode!(input, in_pos);
			let c1 = do_decode!(input, in_pos + 1);
			let c2 = do_decode!(input, in_pos + 2);
			let c3 = do_decode!(input, in_pos + 3);
			let buff = (c0 << 18) | (c1 << 12) | (c2 << 6) | c3;
			output[out_pos] = (buff >> 16) as u8;
			output[out_pos + 1] = (buff >> 8) as u8;
			output[out_pos + 2] = buff as u8;
			in_pos += 4;
			out_pos += 3;
		}
		
		let c0 = do_decode!(input, in_pos);
		let c1 = do_decode!(input, in_pos + 1);
		let mut buff = (c0 << 18) | (c1 << 12);
		output[out_pos] = (buff >> 16) as u8;
		out_pos += 1;
		if input[in_pos + 2] == PADDING
		{
			if input[in_pos + 3] != PADDING
			{
				return Err(DecodeError::Malformed{at: in_pos + 3, value: input[in_pos + 3]});
			}
		}
		else
		{
			buff |= do_decode!(input, in_pos + 2) << 6;
			output[out_pos] = (buff >> 8) as u8;
			out_pos += 1;
			if input[in_pos + 3] != PADDING
			{
				buff |= do_decode!(input, in_pos + 3);
				output[out_pos] = buff as u8;
				out_pos += 1;
			}
		}
		in_pos += 4;
		
		assert_eq!(in_pos, input.len(), "missed input ({in_pos}, expected {})", input.len());
		assert_eq!(out_pos, expect_len, "missed output ({out_pos}, expected {expect_len})");
		Ok(out_pos)
	}
	else {Ok(0)}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError
{
	Malformed{at: usize, value: u8},
	Overflow{need: usize, have: usize},
	Truncated,
	TrailingData{at: usize},
}

impl fmt::Display for DecodeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::Malformed{at, value} => write!(f, "malformed base64 character {value:?} (at {at})"),
			Self::Overflow{need, have} => write!(f, "decoder overflow (need {need}, but only have {have})"),
			Self::Truncated => f.write_str("truncated base64 input stream"),
			Self::TrailingData{at} => write!(f, "trailing data in base64 stream (at {at})"),
		}
	}
}

impl Error for DecodeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError
{
	Overflow{need: usize, have: usize}
}

impl fmt::Display for EncodeError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		match self
		{
			Self::Overflow{need, have} => write!(f, "encoder overflow (need {need}, but only have {have})"),
		}
	}
}

impl Error for EncodeError {}

#[cfg(test)]
mod test
{
	use super::*;
	
	#[test]
	fn validate_chars()
	{
		for (i, &c0) in CHARS.iter().enumerate()
		{
			assert_ne!(c0, PADDING, "padding character in data charset at {i}");
			if i > 0
			{
				for (j, &c1) in CHARS[..i].iter().enumerate()
				{
					assert_ne!(c1, c0, "duplicate data character at {j} and {i}")
				}
			}
		}
	}
	
	#[test]
	fn decode_matches_chars()
	{
		for (i, &c0) in CHARS.iter().enumerate()
		{
			assert_eq!(decode_char(c0), Some(i), "data character {c0} (at {i}) isn't decoded properly");
		}
	}
	
	macro_rules!test_codec
	{
		($func:ident, $input:expr => $expect:expr) =>
		{
			{
				const EMPTY: u8 = 0xC9u8; // arbitrary
				let mut output = [EMPTY; 64];
				let input = $input;
				let expect = $expect;
				assert_eq!($func(input, &mut output), Ok(expect.len()));
				assert_eq!(&output[..expect.len()], expect);
				assert!(output[expect.len()..].iter().all(|&x| x == EMPTY), "output buffer overflow");
			}
		};
	}
	
	#[test]
	fn encoder_success()
	{
		test_codec!(encode, b"Hello Wor" => b"SGVsbG8gV29y");
		test_codec!(encode, b"Hello Worl" => b"SGVsbG8gV29ybA==");
		test_codec!(encode, b"Hello World" => b"SGVsbG8gV29ybGQ=");
		test_codec!(encode, b"Hello World!" => b"SGVsbG8gV29ybGQh");
	}
	
	#[test]
	fn decoder_success()
	{
		test_codec!(decode, b"SGVsbG8gV29y" => b"Hello Wor");
		test_codec!(decode, b"SGVsbG8gV29ybA==" => b"Hello Worl");
		test_codec!(decode, b"SGVsbG8gV29ybGQ=" => b"Hello World");
		test_codec!(decode, b"SGVsbG8gV29ybGQh" => b"Hello World!");
	}
	
	#[test]
	fn encoder_fail()
	{
		let mut output = [0u8; 64];
		assert_eq!(encode(b"Hello Worl", &mut output[..15]), Err(EncodeError::Overflow{need: 16, have: 15}));
		assert_eq!(encode(b"Hello World!", &mut output[..0]), Err(EncodeError::Overflow{need: 16, have: 0}));
	}
	
	#[test]
	fn decoder_fail()
	{
		let mut output = [0u8; 64];
		assert_eq!(decode(b"SGVsbG8gV29ybA==", &mut output[..9]), Err(DecodeError::Overflow{need: 10, have: 9}));
		assert_eq!(decode(b"SGVsbG8gV29ybGQh", &mut output[..11]), Err(DecodeError::Overflow{need: 12, have: 11}));
		assert_eq!(decode(b"SGVsbG8gV29ybA", &mut output), Err(DecodeError::Truncated));
		assert_eq!(decode(b"SGVsbG8gV29yb", &mut output), Err(DecodeError::Truncated));
		assert_eq!(decode(b"SGVsbG8gV29y\n", &mut output), Err(DecodeError::Malformed{at: 12, value: b'\n'}));
		assert_eq!(decode(b"SGVs_bG8gV29y\n", &mut output), Err(DecodeError::Malformed{at: 4, value: b'_'}));
		assert_eq!(decode(b"SGVsbG8gV29ybA==*", &mut output), Err(DecodeError::TrailingData{at: 16}));
		assert_eq!(decode(b"SGVsbG8gV29ybA=*", &mut output), Err(DecodeError::Malformed{at: 15, value: b'*'}));
		assert_eq!(decode(b"SGVsbG8gV29ybA=A", &mut output), Err(DecodeError::Malformed{at: 15, value: b'A'}));
	}
}
