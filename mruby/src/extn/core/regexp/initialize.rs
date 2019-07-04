use std::mem;

use crate::convert::RustBackedValue;
use crate::extn::core::regexp::{Encoding, Options, Regexp};
use crate::sys;
use crate::value::Value;
use crate::warn::MrbWarn;
use crate::Mrb;
use crate::MrbError;

#[derive(Debug)]
pub struct Args {
    pub pattern: Value,
    pub options: Option<Options>,
    pub encoding: Option<Encoding>,
}

impl Args {
    const ARGSPEC: &'static [u8] = b"o|o?o?\0";

    pub fn extract(interp: &Mrb) -> Result<Self, MrbError> {
        let pattern = unsafe { mem::uninitialized::<sys::mrb_value>() };
        let opts = unsafe { mem::uninitialized::<sys::mrb_value>() };
        let has_opts = unsafe { mem::uninitialized::<sys::mrb_bool>() };
        let enc = unsafe { mem::uninitialized::<sys::mrb_value>() };
        let has_enc = unsafe { mem::uninitialized::<sys::mrb_bool>() };
        unsafe {
            sys::mrb_get_args(
                interp.borrow().mrb,
                Self::ARGSPEC.as_ptr() as *const i8,
                &pattern,
                &opts,
                &has_opts,
                &enc,
                &has_enc,
            );
        }
        let has_opts = has_opts != 0;
        let has_enc = has_enc != 0;
        let pattern = Value::new(&interp, pattern);
        let options = if has_opts {
            Some(Options::from_value(&interp, opts)?)
        } else {
            None
        };
        let encoding = if has_enc {
            Some(Encoding::from_value(&interp, enc, false)?)
        } else if has_opts {
            Some(Encoding::from_value(&interp, opts, true)?)
        } else {
            None
        };

        Ok(Self {
            pattern,
            options,
            encoding,
        })
    }
}

pub enum Error {
    Fatal,
    NoImplicitConversionToString,
    Syntax,
    Unicode,
}

pub fn method(interp: &Mrb, slf: sys::mrb_value, args: Args) -> Result<Value, Error> {
    let mut literal_options = args.options.unwrap_or_default();
    let literal_pattern =
        if let Ok(regexp) = unsafe { Regexp::try_from_ruby(interp, &args.pattern) } {
            interp
                .warn("flags ignored when initializing from Regexp")
                .map_err(|_| Error::Fatal)?;
            let borrow = regexp.borrow();
            literal_options = borrow.options;
            borrow.literal_pattern.clone()
        } else {
            let bytes = args
                .pattern
                .try_into::<Vec<u8>>()
                .map_err(|_| Error::NoImplicitConversionToString)?;
            String::from_utf8(bytes).map_err(|_| Error::Unicode)?
        };
    let (pattern, options) = Options::from_pattern(literal_pattern.as_str(), literal_options);
    if let Some(data) = Regexp::new(
        literal_pattern,
        pattern,
        literal_options,
        options,
        args.encoding.unwrap_or_default(),
    ) {
        unsafe {
            data.try_into_ruby(interp, Some(slf))
                .map_err(|_| Error::Fatal)
        }
    } else {
        // Regexp is invalid.
        Err(Error::Syntax)
    }
}