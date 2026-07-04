#![warn(missing_docs)]

//! Unescape the given string.
//! This is the opposite operation of [`std::ascii::escape_default`].

// crates.io
use thiserror::Error as ThisError;

#[cfg(test)] mod test;

/// Unescaper's `Result`.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Unescaper's `Error`.
#[allow(missing_docs)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, ThisError)]
pub enum Error {
	#[error("incomplete str, break at {0}")]
	IncompleteStr(usize),
	#[error("invalid char, {char:?} break at {pos}")]
	InvalidChar { char: char, pos: usize },
	#[error("parse int error, break at {pos}")]
	ParseIntError { source: ::std::num::ParseIntError, pos: usize },
}
use Error::*;

enum LossyEscape {
	Escaped { char: char, len: usize },
	Literal(usize),
}

/// Unescaper struct which holding the chars cache for unescaping.
#[derive(Debug)]
pub struct Unescaper {
	/// [`str`] cache, in reverse order.
	pub chars: Vec<char>,
}
impl Unescaper {
	/// Build a new [`Unescaper`] from the given [`str`].
	pub fn new(s: &str) -> Self {
		Self { chars: s.chars().rev().collect() }
	}

	/// Unescape the given [`str`].
	pub fn unescape(&mut self) -> Result<String> {
		let chars_count = self.chars.len();
		let offset = |mut e, remaining_count| {
			let (IncompleteStr(pos) | InvalidChar { pos, .. } | ParseIntError { pos, .. }) = &mut e;

			*pos += chars_count - remaining_count - 1;

			e
		};
		let mut unescaped = String::new();

		while let Some(c) = self.chars.pop() {
			if c != '\\' {
				unescaped.push(c);

				continue;
			}

			let c = self.chars.pop().ok_or(IncompleteStr(chars_count - self.chars.len() - 1))?;
			let c = match c {
				'b' => '\u{0008}',
				'f' => '\u{000c}',
				'n' => '\n',
				'r' => '\r',
				't' => '\t',
				'v' => '\u{000b}',
				// https://github.com/hack-ink/unescaper/pull/10#issuecomment-1676443635
				//
				// https://www.ecma-international.org/wp-content/uploads/ECMA-404_2nd_edition_december_2017.pdf
				// On page 4 it says: "\/ represents the solidus character (U+002F)."
				'\'' | '\"' | '\\' | '/' => c,
				'u' => self.unescape_unicode_internal().map_err(|e| offset(e, self.chars.len()))?,
				'x' => self.unescape_byte_internal().map_err(|e| offset(e, self.chars.len()))?,
				_ => self.unescape_octal_internal(c).map_err(|e| offset(e, self.chars.len()))?,
			};

			unescaped.push(c);
		}

		Ok(unescaped)
	}

	// pub fn unescape_unicode(&mut self) -> Result<char> {}
	fn unescape_unicode_internal(&mut self) -> Result<char> {
		let c = self.chars.pop().ok_or(Error::IncompleteStr(0))?;
		let mut unicode = String::new();

		// \u + { + regex(d*) + }
		if c == '{' {
			while let Some(n) = self.chars.pop() {
				if n == '}' {
					break;
				}

				unicode.push(n);
			}
		}
		// \u + regex(d{4})
		else {
			// [0, 65536), 16^4
			unicode.push(c);

			for i in 0..3 {
				let c = self.chars.pop().ok_or(IncompleteStr(i))?;

				unicode.push(c);
			}
		}

		char::from_u32(
			u32::from_str_radix(&unicode, 16).map_err(|e| ParseIntError { source: e, pos: 0 })?,
		)
		.ok_or(Error::InvalidChar {
			char: unicode.chars().last().expect("empty unicode will exit earlier; qed"),
			pos: 0,
		})
	}

	// pub fn unescape_byte(&mut self) -> Result<char> {}
	fn unescape_byte_internal(&mut self) -> Result<char> {
		let mut byte = String::new();

		// [0, 256), 16^2
		for i in 0..2 {
			let c = self.chars.pop().ok_or(IncompleteStr(i))?;

			byte.push(c);
		}

		Ok(u8::from_str_radix(&byte, 16).map_err(|e| ParseIntError { source: e, pos: 0 })? as _)
	}

	// pub fn unescape_octal(&mut self) -> Result<char> {}
	fn unescape_octal_internal(&mut self, c: char) -> Result<char> {
		let mut octal = String::new();
		let mut try_push_next = |octal: &mut String| {
			if let Some(c) =
				self.chars.last().cloned().filter(|c| c.is_digit(8)).and_then(|_| self.chars.pop())
			{
				octal.push(c);
			}
		};

		match c {
			// decimal [0, 256) == octal [0, 400)
			// 0 <= first digit < 4
			// \ + regex(d{1,3})
			'0' | '1' | '2' | '3' => {
				octal.push(c);

				(0..2).for_each(|_| try_push_next(&mut octal));
			},
			// \ + regex(d{1,2})
			'4' | '5' | '6' | '7' => {
				octal.push(c);

				try_push_next(&mut octal);
			},
			_ => Err(InvalidChar { char: c, pos: 0 })?,
		}

		Ok(u8::from_str_radix(&octal, 8).map_err(|e| ParseIntError { source: e, pos: 0 })? as _)
	}
}

/// Unescape the given [`str`].
pub fn unescape(s: &str) -> Result<String> {
	Unescaper::new(s).unescape()
}

/// Unescape the given [`str`], preserving malformed escape sequences as written.
///
/// This is useful for user-provided text where valid escape sequences should be interpreted, but
/// malformed escape sequences should not make the whole string fail.
pub fn unescape_lossy(s: &str) -> String {
	let mut unescaped = String::new();
	let mut cursor = 0;

	while cursor < s.len() {
		let remaining = &s[cursor..];
		let c = remaining.chars().next().expect("cursor is in bounds; qed");

		if c != '\\' {
			unescaped.push(c);
			cursor += c.len_utf8();

			continue;
		}

		match parse_lossy_escape(remaining) {
			LossyEscape::Escaped { char, len } => {
				unescaped.push(char);
				cursor += len;
			},
			LossyEscape::Literal(len) => {
				unescaped.push_str(&remaining[..len]);
				cursor += len;
			},
		}
	}

	unescaped
}

fn parse_lossy_escape(s: &str) -> LossyEscape {
	let mut chars = s.char_indices();
	let Some((_, '\\')) = chars.next() else {
		unreachable!("lossy escape parser is only called at escapes")
	};
	let Some((marker_pos, marker)) = chars.next() else {
		return LossyEscape::Literal('\\'.len_utf8());
	};

	match marker {
		'b' => LossyEscape::Escaped { char: '\u{0008}', len: marker_pos + marker.len_utf8() },
		'f' => LossyEscape::Escaped { char: '\u{000c}', len: marker_pos + marker.len_utf8() },
		'n' => LossyEscape::Escaped { char: '\n', len: marker_pos + marker.len_utf8() },
		'r' => LossyEscape::Escaped { char: '\r', len: marker_pos + marker.len_utf8() },
		't' => LossyEscape::Escaped { char: '\t', len: marker_pos + marker.len_utf8() },
		'v' => LossyEscape::Escaped { char: '\u{000b}', len: marker_pos + marker.len_utf8() },
		'\'' | '\"' | '\\' | '/' =>
			LossyEscape::Escaped { char: marker, len: marker_pos + marker.len_utf8() },
		'u' => parse_lossy_unicode_escape(s, marker_pos + marker.len_utf8()),
		'x' => parse_lossy_byte_escape(s, marker_pos + marker.len_utf8()),
		'0'..='7' => parse_lossy_octal_escape(s, marker_pos, marker),
		_ => LossyEscape::Literal(marker_pos + marker.len_utf8()),
	}
}

fn parse_lossy_unicode_escape(s: &str, after_marker: usize) -> LossyEscape {
	if s[after_marker..].starts_with('{') {
		let digits_start = after_marker + '{'.len_utf8();

		if let Some(closing_offset) = s[digits_start..].find('}') {
			let closing = digits_start + closing_offset;
			let digits = &s[digits_start..closing];
			let len = closing + '}'.len_utf8();

			return u32::from_str_radix(digits, 16)
				.ok()
				.and_then(char::from_u32)
				.map_or(LossyEscape::Literal(len), |char| LossyEscape::Escaped { char, len });
		}

		let len = incomplete_braced_unicode_escape_len(s, digits_start);

		if len == s.len() {
			let digits = &s[digits_start..];

			return u32::from_str_radix(digits, 16)
				.ok()
				.and_then(char::from_u32)
				.map_or(LossyEscape::Literal(len), |char| LossyEscape::Escaped { char, len });
		}

		return LossyEscape::Literal(len);
	}

	let Some((digits, len)) = fixed_width_escape_digits(&s[after_marker..], 4) else {
		return LossyEscape::Literal(incomplete_fixed_width_escape_len(s, after_marker, 4));
	};

	u32::from_str_radix(digits, 16).ok().and_then(char::from_u32).map_or(
		LossyEscape::Literal(after_marker + len),
		|char| LossyEscape::Escaped { char, len: after_marker + len },
	)
}

fn parse_lossy_byte_escape(s: &str, after_marker: usize) -> LossyEscape {
	let Some((digits, len)) = fixed_width_escape_digits(&s[after_marker..], 2) else {
		return LossyEscape::Literal(incomplete_fixed_width_escape_len(s, after_marker, 2));
	};

	u8::from_str_radix(digits, 16).map_or(LossyEscape::Literal(after_marker + len), |byte| {
		LossyEscape::Escaped { char: byte as _, len: after_marker + len }
	})
}

fn fixed_width_escape_digits(s: &str, width: usize) -> Option<(&str, usize)> {
	let mut count = 0;

	for (pos, c) in s.char_indices() {
		if c == '\\' {
			break;
		}

		count += 1;

		if count == width {
			let end = pos + c.len_utf8();

			return Some((&s[..end], end));
		}
	}

	None
}

fn incomplete_fixed_width_escape_len(s: &str, after_marker: usize, width: usize) -> usize {
	let mut end = after_marker;

	for (count, (pos, c)) in s[after_marker..].char_indices().enumerate() {
		if c == '\\' || count == width {
			break;
		}

		end = after_marker + pos + c.len_utf8();
	}

	end
}

fn incomplete_braced_unicode_escape_len(s: &str, digits_start: usize) -> usize {
	let mut end = digits_start;

	for (pos, c) in s[digits_start..].char_indices() {
		if c == '\\' {
			break;
		}

		end = digits_start + pos + c.len_utf8();
	}

	end
}

fn parse_lossy_octal_escape(s: &str, marker_pos: usize, marker: char) -> LossyEscape {
	let max_extra_digits = if matches!(marker, '0'..='3') { 2 } else { 1 };
	let mut octal = String::from(marker);
	let mut len = marker_pos + marker.len_utf8();
	let mut end = len;

	for (pos, c) in s[len..].char_indices().take(max_extra_digits) {
		if !c.is_digit(8) {
			break;
		}

		octal.push(c);
		end = len + pos + c.len_utf8();
	}
	len = end;

	u8::from_str_radix(&octal, 8)
		.map_or(LossyEscape::Literal(len), |byte| LossyEscape::Escaped { char: byte as _, len })
}
