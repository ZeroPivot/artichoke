use crate::convert::to_int;
use crate::extn::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Args {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanoseconds: u32,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            year: 0,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            nanoseconds: 0,
        }
    }
}

impl TryConvertMut<&mut [Value], Args> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut args: &mut [Value]) -> Result<Args, Self::Error> {
        // Time args should have a length of 1..=8 or 10. The error does not
        // give a hint that the 10 arg variant is supported however (this is
        // the same in MRI).
        if let 0 | 9 | 11 = args.len() {
            let mut message = br#"wrong number of arguments (given "#.to_vec();
            message.extend(args.len().to_string().bytes());
            message.extend_from_slice(b", expected 1..8)");
            return Err(ArgumentError::from(message).into());
        }

        // Args are in order of year, month, day, hour, minute, second, micros.
        // This is unless there are 10 arguments provided (`Time#to_a` format),
        // at which points it is second, minute, hour, day, month, year.
        if args.len() == 10 {
            args.swap(0, 5);
            args.swap(1, 4);
            args.swap(2, 3);
            // All arguments after position 5 are ignored in the 10 argument
            // variant.
            args = &mut args[..6];
        }

        let mut result = Args::default();

        for (i, &arg) in args.iter().enumerate() {
            match i {
                0 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into::<Option<i64>>(self)?.unwrap();

                    result.year = i32::try_from(arg).map_err(|_| ArgumentError::with_message("year out of range"))?;
                }
                1 => {
                    // TODO: This should support 3 letter month names
                    // as per the docs. https://ruby-doc.org/3.1.2/Time.html#method-c-new
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into::<Option<i64>>(self)?.unwrap();

                    result.month = match u8::try_from(arg) {
                        Ok(month @ 1..=12) => Ok(month),
                        _ => Err(ArgumentError::with_message("mon out of range")),
                    }?;
                }
                2 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into::<Option<i64>>(self)?.unwrap();

                    result.day = match u8::try_from(arg) {
                        Ok(day @ 1..=31) => Ok(day),
                        _ => Err(ArgumentError::with_message("mday out of range")),
                    }?;
                }
                3 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into::<Option<i64>>(self)?.unwrap();

                    result.hour = match u8::try_from(arg) {
                        Ok(hour @ 0..=59) => Ok(hour),
                        _ => Err(ArgumentError::with_message("hour out of range")),
                    }?;
                }
                4 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into::<Option<i64>>(self)?.unwrap();

                    result.minute = match u8::try_from(arg) {
                        Ok(minute @ 0..=59) => Ok(minute),
                        _ => Err(ArgumentError::with_message("min out of range")),
                    }?;
                }
                5 => {
                    // TODO: This should support f64 seconds and drop
                    // the remainder into micros.
                    // ```irb
                    // 3.1.2 > Time.utc(1, 2, 3, 4, 5, 6.1)
                    // => 0001-02-03 04:05:06 56294995342131/562949953421312 UTC
                    // ```
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into::<Option<i64>>(self)?.unwrap();

                    result.second = match u8::try_from(arg) {
                        Ok(second @ 0..=59) => Ok(second),
                        _ => Err(ArgumentError::with_message("sec out of range")),
                    }?;
                }
                6 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into::<Option<i64>>(self)?.unwrap();

                    // Args take a micros parameter, not a nanos value, and
                    // therefore we must multiply the value by 1_000. This is
                    // gaurnateed to fit in a u32.
                    result.nanoseconds = match u32::try_from(arg) {
                        Ok(micros @ 0..=999_999) => Ok(micros * 1000),
                        _ => Err(ArgumentError::with_message("subsecx out of range")),
                    }?;
                }
                7 => {
                    // NOOP
                    // The 8th parameter can be anything, even an error
                    //
                    // ```irb
                    // Time.utc(2022, 1, 1, 0, 0, 0, 0, StandardError)
                    // => 2022-01-01 00:00:00 UTC
                    // ```
                }
                _ => {
                    // The 10 argument variant truncates, and the max length
                    // other variants is 8, so this should always be
                    // unreachable.
                    unreachable!()
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;

    use super::Args;
    use crate::test::prelude::*;

    #[test]
    fn requires_at_least_one_param() {
        let mut interp = interpreter();

        let mut args = vec![];

        let result: Result<Args, Error> = interp.try_convert_mut(args.as_mut_slice());
        let error = result.unwrap_err();

        assert_eq!(error.name(), "ArgumentError");
        assert_eq!(
            error.message().as_bstr(),
            b"wrong number of arguments (given 0, expected 1..8)".as_bstr()
        );
    }

    #[test]
    fn eight_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6, 7, nil]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(5, result.minute);
        assert_eq!(6, result.second);
        assert_eq!(7000, result.nanoseconds);
    }

    #[test]
    fn seven_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6, 7]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(5, result.minute);
        assert_eq!(6, result.second);
        assert_eq!(7000, result.nanoseconds);
    }

    #[test]
    fn six_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(5, result.minute);
        assert_eq!(6, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn five_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(5, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn four_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(0, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn three_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(0, result.hour);
        assert_eq!(0, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn two_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(1, result.day);
        assert_eq!(0, result.hour);
        assert_eq!(0, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn one_param() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(1, result.month);
        assert_eq!(1, result.day);
        assert_eq!(0, result.hour);
        assert_eq!(0, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn subsec_is_valid_micros_not_nanos() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 1, 1, 0, 0, 0, 1]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        let nanos = result.nanoseconds;
        assert_eq!(1000, nanos);

        let args = interp.eval(b"[2022, 1, 1, 0, 0, 0, 999_999]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        let nanos = result.nanoseconds;
        assert_eq!(999_999_000, nanos);
    }

    #[test]
    fn subsec_does_not_wrap_around() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 1, 1, 0, 0, 0, -1]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();
        assert_eq!(error.message().as_bstr(), b"subsecx out of range".as_bstr());

        let args = interp.eval(b"[2022, 1, 1, 0, 0, 0, 1_000_000]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();
        assert_eq!(error.message().as_bstr(), b"subsecx out of range".as_bstr());
    }

    #[test]
    fn fractional_seconds_return_nanos() {}

    #[test]
    fn nine_args_not_supported() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6, 7, nil, 0]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();

        assert_eq!(
            error.message().as_bstr(),
            b"wrong number of arguments (given 9, expected 1..8)".as_bstr()
        );
        assert_eq!(error.name(), "ArgumentError");
    }

    #[test]
    fn ten_args_changes_unit_order() {
        let mut interp = interpreter();

        let args = interp.eval(b"[1, 2, 3, 4, 5, 2022, nil, nil, nil, nil]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();

        assert_eq!(1, result.second);
        assert_eq!(2, result.minute);
        assert_eq!(3, result.hour);
        assert_eq!(4, result.day);
        assert_eq!(5, result.month);
        assert_eq!(2022, result.year);
    }

    #[test]
    fn eleven_args_is_too_many() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6, 7, nil, 0, 0, 0]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();

        assert_eq!(
            error.message().as_bstr(),
            b"wrong number of arguments (given 11, expected 1..8)".as_bstr()
        );
        assert_eq!(error.name(), "ArgumentError");
    }
}
