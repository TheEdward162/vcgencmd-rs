use thiserror::Error;

// input:
// "field1=value1 field2=value2"
//
// `field` is an arbitrary string (e.g. "frequency(48)")
// `value` is either:
// * string delimited by `"`
// * floating point value (with a possible text suffix to be stripped) (e.g. 45.3'C)
// * integer value

#[derive(Error, Debug)]
pub enum ParseFieldError<E: std::error::Error + 'static> {
	#[error("Invalid format")]
	InvalidFormatError,
	#[error("Invalid value format: {0}")]
	ConversionError(#[from] E)
}

pub trait ParseFieldType<'a>: Sized {
	type Error: std::error::Error;

	fn parse(source: &'a str) -> Result<(&'a str, Self), Self::Error>;
}

pub trait IntFromStrRadix: Sized {
	fn from_str_radix(source: &str, radix: u32) -> Result<Self, std::num::ParseIntError>;
}
pub struct IntRadix<I: IntFromStrRadix, const RADIX: u32>(pub I);
impl<'a, I: IntFromStrRadix, const RADIX: u32> ParseFieldType<'a> for IntRadix<I, RADIX> {
	type Error = std::num::ParseIntError;

	fn parse(source: &'a str) -> Result<(&'a str, Self), Self::Error> {
		let end = source
			.find(|ch: char| !(ch == '-' || ch == '+' || ch.is_digit(RADIX)))
			.unwrap_or(source.len());

		let value_str = &source[.. end];
		let value = I::from_str_radix(value_str, RADIX)?;

		Ok((&source[end ..], IntRadix(value)))
	}
}
macro_rules! impl_from_str_radix {
	(
		$(
			$integer: ty
		)+
	) => {
		$(
			impl IntFromStrRadix for $integer {
				fn from_str_radix(source: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
					Self::from_str_radix(source, radix)
				}
			}

			impl<'a> ParseFieldType<'a> for $integer {
				type Error = std::num::ParseIntError;

				fn parse(source: &'a str) -> Result<(&'a str, Self), Self::Error> {
					IntRadix::<Self, 10>::parse(source).map(|(s, r)| (s, r.0))
				}
			}
		)+
	};
}
impl_from_str_radix!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128);

impl<'a> ParseFieldType<'a> for f32 {
	type Error = std::num::ParseFloatError;

	fn parse(source: &'a str) -> Result<(&'a str, Self), Self::Error> {
		let end = source
			.find(|ch: char| !(ch == '-' || ch == '+' || ch == '.' || ch.is_digit(10)))
			.unwrap_or(source.len());

		let value_str = &source[.. end];
		let value = value_str.parse::<Self>()?;

		Ok((&source[end ..], value))
	}
}

#[derive(Error, Debug)]
#[error("string values must be delimited by \"")]
pub struct ParseStrError;
impl<'a> ParseFieldType<'a> for &'a str {
	type Error = ParseStrError;

	fn parse(mut source: &'a str) -> Result<(&'a str, Self), Self::Error> {
		if !source.starts_with('"') {
			return Err(ParseStrError)
		}
		source = &source[1 ..];

		let end = source.find('"').ok_or(ParseStrError)?;

		let value = &source[.. end];
		source = &source[end + 1 ..];

		Ok((source, value))
	}
}

pub fn parse_field_simple<'a, T: ParseFieldType<'a>>(
	source: &'a str,
	key: &str
) -> Result<(&'a str, T), ParseFieldError<T::Error>> {
	parse_field(source, key, None, None)
}

pub fn parse_field<'a, T: ParseFieldType<'a>>(
	mut source: &'a str,
	key: &str,
	value_prefix: Option<&str>,
	value_suffix: Option<&str>
) -> Result<(&'a str, T), ParseFieldError<T::Error>> {
	log::trace!(
		"Parsing field: source: {}, key: {}, prefix: {:?}, postfix: {:?}",
		source,
		key,
		value_prefix,
		value_suffix
	);

	// trim whitespace
	source = source.trim_start();

	// check key
	if !source.starts_with(key) {
		return Err(ParseFieldError::InvalidFormatError)
	}
	source = &source[key.len() ..];

	// check equals sign
	if !source.starts_with('=') {
		return Err(ParseFieldError::InvalidFormatError)
	}
	source = &source[1 ..];

	// prefix
	if let Some(prefix) = value_prefix {
		if !source.starts_with(prefix) {
			return Err(ParseFieldError::InvalidFormatError)
		}
		source = &source[prefix.len() ..];
	}

	// value
	let (new_source, value) = T::parse(source)?;
	source = new_source;

	// suffix
	if let Some(suffix) = value_suffix {
		if !source.starts_with(suffix) {
			return Err(ParseFieldError::InvalidFormatError)
		}
		source = &source[suffix.len() ..];
	}

	Ok((source, value))
}

#[cfg(test)]
mod test {
	use super::{parse_field, parse_field_simple, IntRadix};

	#[test]
	fn parses_i32_field() {
		let source = " value=-1234";

		let (new_source, value) = parse_field_simple::<i32>(source, "value").unwrap();

		assert_eq!(value, -1234);
		assert_eq!(
			unsafe { source.as_ptr().add(source.len()) },
			new_source.as_ptr()
		);
	}

	#[test]
	fn parses_u32_field_with_radix() {
		let source = " value=10  ";

		let (new_source, value) = parse_field_simple::<IntRadix<u32, 2>>(source, "value").unwrap();

		assert_eq!(value.0, 2);
		assert_eq!(
			unsafe { source.as_ptr().add(" value=10".len()) },
			new_source.as_ptr()
		);
	}

	#[test]
	fn parses_f32_field() {
		let source = "value(12)=12.3";

		let (new_source, value) = parse_field_simple::<f32>(source, "value(12)").unwrap();

		assert_eq!(value, 12.3);
		assert_eq!(
			unsafe { source.as_ptr().add(source.len()) },
			new_source.as_ptr()
		);
	}

	#[test]
	fn parses_prefixed_field() {
		let source = "value=0x12 value2=1";

		let (new_source, value) = parse_field::<u64>(source, "value", Some("0x"), None).unwrap();

		assert_eq!(value, 12);
		assert_eq!(
			unsafe { source.as_ptr().add("value=0x12".len()) },
			new_source.as_ptr()
		);
	}

	#[test]
	fn parses_suffixed_field() {
		let source = "value(12)=12.3'C value2=1";

		let (new_source, value) =
			parse_field::<f32>(source, "value(12)", None, Some("'C")).unwrap();

		assert_eq!(value, 12.3);
		assert_eq!(
			unsafe { source.as_ptr().add("value(12)=12.3'C".len()) },
			new_source.as_ptr()
		);
	}

	#[test]
	fn parses_prefixed_and_suffixed_field() {
		let source = "value(12)=0x12.3'C value2=1";

		let (new_source, value) =
			parse_field::<f32>(source, "value(12)", Some("0x"), Some("'C")).unwrap();

		assert_eq!(value, 12.3);
		assert_eq!(
			unsafe { source.as_ptr().add("value(12)=0x12.3'C".len()) },
			new_source.as_ptr()
		);
	}

	#[test]
	fn parses_str_field() {
		let source = "value=\"one, two, three, four\"";

		let (new_source, value) = parse_field_simple::<&str>(source, "value").unwrap();

		assert_eq!(value, "one, two, three, four");
		assert_eq!(
			unsafe { source.as_ptr().add(source.len()) },
			new_source.as_ptr()
		);
	}

	#[test]
	fn parses_field_with_correct_end() {
		let source = "value=\"one, two, three, four\" value2=1";

		let (new_source, value) = parse_field_simple::<&str>(source, "value").unwrap();

		assert_eq!(value, "one, two, three, four");
		assert_eq!(
			unsafe { source.as_ptr().add("value=\"one, two, three, four\"".len()) },
			new_source.as_ptr()
		);
	}
}
