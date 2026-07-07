use std::error::Error as _;

use proptest::{collection, prelude};

use crate::Error::{IncompleteStr, InvalidChar};

macro_rules! unescape_assert_eq {
	($l:expr, $r:expr) => {
		assert_eq!(crate::unescape($l).unwrap(), $r);
	};
}

macro_rules! unescape_assert_err {
	($l:expr, $r:expr) => {
		assert_eq!(crate::unescape($l).unwrap_err(), $r);
	};
}

macro_rules! unescape_assert_err_str {
	($s:expr, $e:expr, $e_src:expr) => {{
		let e = crate::unescape($s).unwrap_err();

		assert_eq!(e.to_string(), $e);
		assert_eq!(e.source().unwrap().to_string(), $e_src);
	}};
}

macro_rules! unescape_assert_invalid_pos {
	($l:expr, $r:expr) => {
		let InvalidChar { pos, .. } = crate::unescape($l).unwrap_err() else {
			panic!("expected invalid char error");
		};

		assert_eq!(pos, $r);
	};
}

proptest::proptest! {
	#[test]
	fn escape_default_round_trips_for_chars(c in prelude::any::<char>()) {
		let escaped = c.escape_default().collect::<String>();

		proptest::prop_assert_eq!(crate::unescape(&escaped).unwrap(), c.to_string());
	}

	#[test]
	fn escape_default_round_trips_for_strings(
		chars in collection::vec(prelude::any::<char>(), 0..64)
	) {
		let source = chars.into_iter().collect::<String>();
		let escaped = source.escape_default().collect::<String>();

		proptest::prop_assert_eq!(crate::unescape(&escaped).unwrap(), source);
	}
}

#[test]
fn strict_errors() {
	unescape_assert_err!(r"\", IncompleteStr(0));
	unescape_assert_err!(r"\x", IncompleteStr(1));
	unescape_assert_err!(r"\x0", IncompleteStr(3));
	unescape_assert_err!(r"\0\", IncompleteStr(2));
	unescape_assert_err!(r"\u{", IncompleteStr(2));
	unescape_assert_err!(r"\u{12", IncompleteStr(4));
	unescape_assert_err!(r"\{}", InvalidChar { char: '{', pos: 1 });
	unescape_assert_err!(r"\0\{}", InvalidChar { char: '{', pos: 3 });
	unescape_assert_invalid_pos!(r"\u{D800}", 7);
	unescape_assert_invalid_pos!(r"\u{110000}", 9);
	unescape_assert_err!("é\\q", InvalidChar { char: 'q', pos: 2 });
	unescape_assert_err_str!(
		r"\u{}",
		"parse int error, break at 3",
		"cannot parse integer from empty string"
	);
	unescape_assert_err_str!(
		r"\u{g}",
		"parse int error, break at 4",
		"invalid digit found in string"
	);
	unescape_assert_err_str!(
		r"\xzz",
		"parse int error, break at 3",
		"invalid digit found in string"
	);
	unescape_assert_err_str!(
		r"\0\u{g}",
		"parse int error, break at 6",
		"invalid digit found in string"
	);
}

#[test]
fn unescape_unicode() {
	unescape_assert_eq!(r"\u0000", "\0");
	unescape_assert_eq!(r"\u0001", "\u{0001}");
	unescape_assert_eq!(r"\u0009", "\t");
	unescape_assert_eq!(r"\u000a", "\n");
	unescape_assert_eq!(r"\u007f", "\u{007f}");
	unescape_assert_eq!(r"\u0080", "\u{0080}");
	unescape_assert_eq!(r"\uffff", "\u{ffff}");
	unescape_assert_eq!(r"\u0000XavierJane", "\0XavierJane");
	unescape_assert_eq!(r"\u{0}", "\0");
	unescape_assert_eq!(r"\u{1}", "\u{0001}");
	unescape_assert_eq!(r"\u{9}", "\t");
	unescape_assert_eq!(r"\u{a}", "\n");
	unescape_assert_eq!(r"\u{7f}", "\u{007f}");
	unescape_assert_eq!(r"\u{80}", "\u{0080}");
	unescape_assert_eq!(r"\u{ffff}", "\u{ffff}");
	unescape_assert_eq!(r"\u{10ffff}", "\u{10ffff}");
	unescape_assert_eq!(r"\u{1F600}", "\u{1F600}");
	unescape_assert_eq!(r"\u{1F600}", "😀");
	unescape_assert_eq!(r"\u{0}XavierJane", "\0XavierJane");
}

#[test]
fn unescape_byte() {
	unescape_assert_eq!(r"\x00", "\x00");
	unescape_assert_eq!(r"\x01", "\x01");
	unescape_assert_eq!(r"\x09", "\t");
	unescape_assert_eq!(r"\x0a", "\n");
	unescape_assert_eq!(r"\x7f", "\x7f");
	unescape_assert_eq!(r"\x80", "\u{0080}");
	unescape_assert_eq!(r"\xff", "\u{00ff}");
	unescape_assert_eq!(r"\x00XavierJane", "\x00XavierJane");
}

#[test]
fn unescape_octal() {
	unescape_assert_eq!(r"\0", "\0");
	unescape_assert_eq!(r"\00", "\0");
	unescape_assert_eq!(r"\000", "\0");
	unescape_assert_eq!(r"\7", "\u{0007}");
	unescape_assert_eq!(r"\11", "\t");
	unescape_assert_eq!(r"\12", "\n");
	unescape_assert_eq!(r"\177", "\u{007f}");
	unescape_assert_eq!(r"\200", "\u{0080}");
	unescape_assert_eq!(r"\377", "\u{00ff}");
	// Leading 4..7 consumes at most one following octal digit; the next digit remains literal.
	unescape_assert_eq!(r"\400", " 0");
	unescape_assert_eq!(r"\777", "?7");
	unescape_assert_eq!(r"\0XavierJane", "\0XavierJane");
}

#[test]
fn unescape_special_symbols() {
	unescape_assert_eq!(r"\b", "\u{0008}");
	unescape_assert_eq!(r"\f", "\u{000c}");
	unescape_assert_eq!(r"\n", "\n");
	unescape_assert_eq!(r"\r", "\r");
	unescape_assert_eq!(r"\t", "\t");
	unescape_assert_eq!(r"\v", "\u{000b}");
	unescape_assert_eq!(r"\'", "\'");
	unescape_assert_eq!(r#"\""#, "\"");
	unescape_assert_eq!(r"\\", "\\");
	unescape_assert_eq!(r"\/", "/");
}

#[test]
fn unescape_lossy() {
	assert_eq!(crate::unescape_lossy(r"a\nb"), "a\nb");
	assert_eq!(crate::unescape_lossy(r"a\qb\nc"), "a\\qb\nc");
	assert_eq!(crate::unescape_lossy(r"\u{g}\n"), "\\u{g}\n");
	assert_eq!(crate::unescape_lossy(r"\u{}"), r"\u{}");
	assert_eq!(crate::unescape_lossy(r"\u{é}"), r"\u{é}");
	assert_eq!(crate::unescape_lossy(r"\u{110000}\n"), "\\u{110000}\n");
	assert_eq!(crate::unescape_lossy(r"x\u{a"), r"x\u{a");
	assert_eq!(crate::unescape_lossy(r"\u{a"), r"\u{a");
	assert_eq!(crate::unescape_lossy(r"\u{1F600"), r"\u{1F600");
	assert_eq!(crate::unescape_lossy(r"\u{12\n"), "\\u{12\n");
	assert_eq!(crate::unescape_lossy(r"\u12\n"), "\\u12\n");
	assert_eq!(crate::unescape_lossy(r"\x0a\xzz"), "\n\\xzz");
	assert_eq!(crate::unescape_lossy(r"\x"), r"\x");
	assert_eq!(crate::unescape_lossy(r"\x0"), r"\x0");
	assert_eq!(crate::unescape_lossy(r"abc\"), "abc\\");
	assert_eq!(crate::unescape_lossy(r"\8"), r"\8");
	assert_eq!(crate::unescape_lossy(r"\377"), "\u{00ff}");
}
