use std::fmt::Debug;
use sal_core::{dbg::Dbg, error::Error};
use super::message::{Bytes, MessageParse};
///
/// Extracting `Data` field from the input bytes fixed length
pub struct FindField<FieldIn, FieldOut, Out> {
    dbg: Dbg,
    size: usize,
    field: Box<dyn MessageParse<FieldIn, FieldOut, Bytes>>,
    field_data: Option<(FieldIn, FieldOut)>,
    from_bytes: Box<dyn Fn(&Dbg, &[u8]) -> Result<Option<Out>, Error>>,
    remainder: Bytes,
}
//
//
impl<FieldIn, FieldOut, Out> FindField<FieldIn, FieldOut, Out> {
    ///
    /// Returns [FindField] new instance
    /// - `size` - Field length in the bytes
    pub fn new(parent: impl Into<String>, size: usize, from_bytes: impl Fn(&Dbg, &[u8]) -> Result<Option<Out>, Error> + 'static, field: impl MessageParse<FieldIn, FieldOut, Bytes> + 'static) -> Self {
        let dbg = Dbg::new(parent, format!("FindField(size {size})"));
        if size == 0 {
            panic!("{dbg}.new | Size should be >= 1");
        }
        Self {
            size,
            from_bytes: Box::new(from_bytes),
            field: Box::new(field),
            field_data: None,
            remainder: Vec::with_capacity(size - 1),
            dbg,
        }
    }
    ///
    /// Returns T of specified bytes length
    fn convert(&mut self, remainder: Vec<u8>) -> Result<(Out, Bytes), Error> {
        let mut e = Error::new(&self.dbg, "");
        if remainder.len() >= self.size {
            match remainder
                .windows(self.size)
                .enumerate()
                .find_map(|(i, bytes)| {
                    match (self.from_bytes)(&self.dbg, bytes) {
                        Ok(data) => match data {
                            Some(data) => if remainder.len() >= self.size + i {
                                Some((data, remainder[(self.size + i)..].to_vec()))
                            } else {
                                Some((data, vec![]))
                            },
                            None => None,
                        }
                        Err(err) => {
                            e = err;
                            None
                        },
                    }
                }) {
                    Some(val) => {
                        self.reset();
                        Ok(val)
                    }
                    None => {
                        match remainder.get((remainder.len() - self.size)..) {
                            Some(r) => {
                                self.remainder = r.to_vec();
                            }
                            None => self.remainder = remainder[1..].to_vec(),
                        }
                        // self.remainder = remainder[(remainder.len() - self.size + 1)..].to_vec();
                        Err(Error::new(&self.dbg, "parse").pass_with("FromBytes error", e))
                    }
                }
        } else {
            self.remainder = remainder;
            Err(Error::new(&self.dbg, "parse").err("Take error"))
        }
    }
    ///
    /// Resets state to the initial
    fn reset(&mut self) {
        self.field_data = None;
    }
}
//
//
impl<FieldIn: Copy + Debug, FieldOut: Copy + Debug, Out: Debug> MessageParse<(FieldIn, FieldOut), Out, Bytes> for FindField<FieldIn, FieldOut, Out> {
    ///
    /// Extracting `Data` field from the input bytes
    /// - returns `Id`, `Kind`, `Size` & `Bytes` following by the `Size`
    /// - call this method multiple times, until the end of message
    fn parse(&mut self, bytes: Bytes) -> Result<((FieldIn, FieldOut), Out, Bytes), Error> {
        let error = Error::new(&self.dbg, "parse");
        // let dbg = self.dbg.clone();
        let remainder = [std::mem::take(&mut self.remainder), bytes].concat();
        // log::debug!("{dbg}.parse | remainder: {:?}", remainder);
        match self.field_data {
            Some((din, dout)) => match self.convert(remainder) {
                Ok((data, remainder)) => {
                    // log::debug!("{}.parse | Field exist | din: {:?}, dout: {:?}, data: {:?}, remainder: {:?}", dbg, din, dout, data, remainder);
                    Ok(((din, dout), data, remainder))
                }
                Err(err) => Err(error.pass(err)),
            }
            None => {
                match self.field.parse(remainder) {
                    Ok((din, dout, remainder)) => {
                        match self.convert(remainder) {
                            Ok((data, remainder)) => {
                                // log::debug!("{}.parse | Field parsed | din: {:?}, dout: {:?}, data: {:?}, remainder: {:?}", dbg, din, dout, data, remainder);
                                Ok(((din, dout), data, remainder))
                            }
                            Err(err) => {
                                self.field_data = Some((din, dout));
                                Err(error.pass(err))
                            }
                        }
                    }
                    Err(err) => Err(error.pass(err)),
                }
            }
        }
    }
}
