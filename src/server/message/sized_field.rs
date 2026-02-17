use std::fmt::Debug;
use sal_core::{dbg::Dbg, error::Error};
use super::message::{Bytes, MessageParse};
///
/// Extracting `Data` field from the input bytes of calculated length
pub struct SizedField<FieldIn, FieldOut, Out> {
    dbg: Dbg,
    size: Box<dyn Fn(&FieldIn, &FieldOut) -> usize>,
    field: Box<dyn MessageParse<FieldIn, FieldOut, Bytes>>,
    field_data: Option<(FieldIn, FieldOut)>,
    from_bytes: Box<dyn Fn(&Dbg, &[u8]) -> Result<Out, Error>>,
    remainder: Bytes,
}
//
//
impl<FieldIn, FieldOut, Out> SizedField<FieldIn, FieldOut, Out> {
    ///
    /// Returns [SizedField] new instance
    /// - `size` - Field length in the bytes calculated from previous fields
    pub fn new(parent: impl Into<String>, size: impl Fn(&FieldIn, &FieldOut) -> usize + 'static, from_bytes: impl Fn(&Dbg, &[u8]) -> Result<Out, Error> + 'static, field: impl MessageParse<FieldIn, FieldOut, Bytes> + 'static) -> Self {
        Self {
            size: Box::new(size),
            from_bytes: Box::new(from_bytes),
            field: Box::new(field),
            field_data: None,
            remainder: vec![],
            dbg: Dbg::new(parent, format!("SizedField")),
        }
    }
    ///
    /// Returns T of specified bytes length
    fn convert(&mut self, remainder: Vec<u8>, size: usize) -> Result<(Out, Bytes), Error> {
        if remainder.len() >= size {
            let bytes = &remainder[..size];
            // log::debug!("{}.parse | Bytes from remainder[{}]: {:?}", self.dbg, size, bytes);
            self.reset();
            match (self.from_bytes)(&self.dbg, bytes) {
                Ok(data) => if remainder.len() >= size {
                    Ok((data, remainder[size..].to_vec()))
                } else {
                    Ok((data, vec![]))
                }
                Err(err) => Err(Error::new(&self.dbg, "parse").pass_with("FromBytes error", err)),
            }
        } else {
            self.remainder.extend(remainder);
            Err(Error::new(&self.dbg, "parse").err("Take error"))
        }
    }
    ///
    /// Resets state to the initial
    fn reset(&mut self) {
        self.field_data = None;
        self.remainder = vec![];
    }
}
//
//
impl<FieldIn: Copy + Debug, FieldOut: Copy + Debug, Out: Debug> MessageParse<(FieldIn, FieldOut), Out, Bytes> for SizedField<FieldIn, FieldOut, Out> {
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
            Some((din, dout)) => match self.convert(remainder, (self.size)(&din, &dout)) {
                Ok((data, remainder)) => {
                    // log::debug!("{}.parse | Field exist | din: {:?}, dout: {:?}, data: {:?}, remainder: {:?}", dbg, din, dout, data, remainder);
                    Ok(((din, dout), data, remainder))
                }
                Err(err) => Err(error.pass_with("Field value from bytes error", err)),
            }
            None => {
                match self.field.parse(remainder) {
                    Ok((din, dout, remainder)) => match self.convert(remainder, (self.size)(&din, &dout)) {
                        Ok((data, remainder)) => {
                            // log::debug!("{}.parse | Field parsed | din: {:?}, dout: {:?}, data: {:?}, remainder: {:?}", dbg, din, dout, data, remainder);
                            Ok(((din, dout), data, remainder))
                        }
                        Err(err) => {
                            self.field_data = Some((din, dout));
                            Err(error.pass_with("Field value from bytes error", err))
                        }
                    }
                    Err(err) => Err(error.pass(err)),
                }
            }
        }
    }
}
