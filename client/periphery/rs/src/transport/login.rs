use anyhow::{Context, anyhow};
use encoding::{
  CastBytes, Decode, Encode, EncodedResponse, impl_cast_bytes_vec,
  impl_from_for_wrapper,
};
use mogh_pki::SpkiPublicKey;
use strum::EnumDiscriminants;

use crate::transport::{EncodedTransportMessage, TransportMessage};

#[derive(Debug)]
pub struct EncodedLoginMessage(
  EncodedResponse<InnerEncodedLoginMessage>,
);

impl_from_for_wrapper!(
  EncodedLoginMessage,
  EncodedResponse<InnerEncodedLoginMessage>
);
impl_cast_bytes_vec!(EncodedLoginMessage, EncodedResponse);

/// ```markdown
/// | -- u8[] -- | --------- u8 ------------ |
/// | <CONTENTS> | LoginMessageVariant |
/// ```
#[derive(Clone, Debug)]
pub struct InnerEncodedLoginMessage(Vec<u8>);

impl_cast_bytes_vec!(InnerEncodedLoginMessage, Vec);

#[derive(Clone, EnumDiscriminants)]
#[strum_discriminants(name(LoginMessageVariant))]
pub enum LoginMessage {
  /// At the end of every login flow,
  /// Send a success message
  Success,
  /// Every handshake includes a random 32 byte nonce
  /// to identify the connection.
  Nonce([u8; 32]),
  /// Bytes that are part of the noise handshake.
  Handshake(Vec<u8>),
  /// Used during Periphery -> Core connections.
  /// Core must let Periphery know which flow to use
  /// before the handshake is started, so it can use
  /// the onboarding key.
  OnboardingFlow(bool),
  /// The onboarding flow requires Periphery to send
  /// over its public key to initialize the Server with
  /// allowed connection.
  PublicKey(SpkiPublicKey),
  /// Used during Core -> Periphery connections.
  /// If Periphery hasn't set `core_public_keys`,
  /// will fall back to passkey auth
  /// for backward compatability with v1
  V1PasskeyFlow(bool),
  /// Core will send the passkey to Periphery to validate
  /// in the V1PasskeyLogin flow.
  V1Passkey(Vec<u8>),
}

impl Encode<EncodedTransportMessage> for LoginMessage {
  fn encode(self) -> EncodedTransportMessage {
    let variant: LoginMessageVariant = (&self).into();
    let variant_byte = variant.as_byte();
    let mut bytes = match self {
      LoginMessage::Success => Vec::new(),
      LoginMessage::Nonce(nonce) => nonce.to_vec(),
      LoginMessage::Handshake(bytes) => bytes,
      LoginMessage::OnboardingFlow(onboarding_flow) => {
        let byte = if onboarding_flow { 1 } else { 0 };
        vec![byte]
      }
      LoginMessage::PublicKey(spki_public_key) => {
        spki_public_key.into_inner().into()
      }
      LoginMessage::V1PasskeyFlow(passkey_flow) => {
        let byte = if passkey_flow { 1 } else { 0 };
        vec![byte]
      }
      LoginMessage::V1Passkey(bytes) => bytes,
    };
    bytes.push(variant_byte);
    let inner = InnerEncodedLoginMessage(bytes);
    let res = Ok(inner).encode();
    TransportMessage::Login(EncodedLoginMessage(res)).encode()
  }
}

impl Decode<LoginMessage> for EncodedLoginMessage {
  fn decode(self) -> anyhow::Result<LoginMessage> {
    let mut bytes = self
      .0
      .decode()?
      .context("Should not receive Pending (2) Response message")?
      .into_vec();

    let variant_byte = bytes
      .pop()
      .context("Failed to parse login message | Bytes are empty")?;

    let variant = LoginMessageVariant::from_byte(variant_byte)?;

    use LoginMessageVariant::*;
    let message = match variant {
      Success => LoginMessage::Success,

      Nonce => LoginMessage::Nonce(
        bytes
          .try_into()
          .map_err(|_| anyhow!("Invalid connection nonce"))?,
      ),

      Handshake => LoginMessage::Handshake(bytes),

      OnboardingFlow => {
        let onboarding_flow = match bytes.as_slice() {
          &[0] => false,
          &[1] => true,
          other => {
            return Err(anyhow!(
              "Got unrecognized LoginMessage OnboardingFlow bytes: {other:?}"
            ));
          }
        };
        LoginMessage::OnboardingFlow(onboarding_flow)
      }

      PublicKey => {
        if bytes.is_empty() {
          return Err(anyhow!(
            "Got empty LoginMessage OnboardingFlow PublicKey bytes"
          ));
        }
        let public_key = String::from_utf8(bytes)
          .context("Public key is not valid utf-8")?;
        LoginMessage::PublicKey(SpkiPublicKey::from(public_key))
      }

      // V1
      V1PasskeyFlow => {
        let passkey_login = match bytes.as_slice() {
          &[0] => false,
          &[1] => true,
          other => {
            return Err(anyhow!(
              "Got unrecognized LoginMessage V1PasskeyLogin bytes: {other:?}"
            ));
          }
        };
        LoginMessage::V1PasskeyFlow(passkey_login)
      }

      V1Passkey => {
        if bytes.is_empty() {
          return Err(anyhow!(
            "Got empty LoginMessage V1Passkey bytes"
          ));
        }
        LoginMessage::V1Passkey(bytes)
      }
    };

    Ok(message)
  }
}

impl LoginMessageVariant {
  pub fn from_byte(byte: u8) -> anyhow::Result<Self> {
    use LoginMessageVariant::*;
    let variant = match byte {
      0 => Success,
      1 => Nonce,
      2 => Handshake,
      3 => OnboardingFlow,
      4 => PublicKey,
      // V1
      5 => V1PasskeyFlow,
      6 => V1Passkey,
      other => {
        return Err(anyhow!(
          "Got unrecognized LoginMessageVariant byte: {other}"
        ));
      }
    };
    Ok(variant)
  }

  pub fn as_byte(self) -> u8 {
    use LoginMessageVariant::*;
    match self {
      Success => 0,
      Nonce => 1,
      Handshake => 2,
      OnboardingFlow => 3,
      PublicKey => 4,
      // V1
      V1PasskeyFlow => 5,
      V1Passkey => 6,
    }
  }
}
