//! Интерфес для парсинга json от  АПИ сервера

use sal_core::error::Error;

/// Интерфес для парсинга json от  АПИ сервера,  
/// проверяет наличие строки с ошибкой
pub trait IFromJson {
    /// Строка с ошибкой если она пришла с сервера
    fn error(&self) -> Option<&String>;
    /// Парсинг данных из json строки
    fn parse<'a>(src: &'a [u8]) -> Result<Self, Error> where Self: Sized + serde::Deserialize<'a> {
        let error = Error::new("IFromJson", "parse");
        match serde_json::from_slice::<Self>(src) {
            Ok(res) => {
                if let Some(err) = res.error() {
                    if !err.is_empty() {
                        return Err(error.pass(err));
                    }
                }
                Ok(res)
            }
            Err(err) => Err(error.pass(err.to_string())),
        }
    }
}
