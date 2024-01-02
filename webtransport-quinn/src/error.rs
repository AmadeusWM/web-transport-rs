use thiserror::Error;

// #[derive(Clone, Error, Debug)]
// pub enum SendDatagramError {
//     #[error("Unsupported peer")]
//     UnsupportedPeer,

//     #[error("Datagram support Disabled by peer")]
//     DatagramSupportDisabled,

//     #[error("Datagram Too large")]
//     TooLarge,

//     #[error("Session errorr: {0}")]
//     SessionError(#[from] SessionError),
// }

// impl From<quinn::SendDatagramError> for SendDatagramError {
//     fn from(value: quinn::SendDatagramError) -> Self {
//          match value {
//              quinn::SendDatagramError::UnsupportedByPeer => SendDatagramError::UnsupportedPeer,
//              quinn::SendDatagramError::Disabled => SendDatagramError::DatagramSupportDisabled,
//              quinn::SendDatagramError::TooLarge => SendDatagramError::TooLarge,
//              quinn::SendDatagramError::ConnectionLost(e) => SendDatagramError::SessionError(e.into()),
//          }
//     }
// }
