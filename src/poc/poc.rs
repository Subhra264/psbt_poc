use super::input::Input;
use super::output::Output;

/// A Partially Signed Transaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "actual_serde"))]
pub struct PartiallySignedTransactionInner {
    /// The unsigned transaction, scriptSigs and witnesses for each input must be empty.
    pub unsigned_tx: Option<Transaction>,
    /// The version number of this PSBT. If omitted, the version number is 0.
    /// See https://github.com/rust-bitcoin/rust-bitcoin/pull/1218
    pub version: Version,
    /// A global map from extended public keys to the used key fingerprint and
    /// derivation path as defined by BIP 32.
    pub xpub: BTreeMap<ExtendedPubKey, KeySource>,
    /// Global proprietary key-value pairs.
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::serde_utils::btreemap_as_seq_byte_values")
    )]
    pub proprietary: BTreeMap<raw::ProprietaryKey, Vec<u8>>,
    /// Unknown global key-value pairs.
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::serde_utils::btreemap_as_seq_byte_values")
    )]
    pub unknown: BTreeMap<raw::Key, Vec<u8>>,

    /// The corresponding key-value map for each input in the unsigned transaction.
    pub inputs: Vec<Input>,
    /// The corresponding key-value map for each output in the unsigned transaction.
    pub outputs: Vec<Output>,
}

pub struct Psbt {
    inner: PartiallySignedTransactionInner,
}

impl Psbt {
    pub fn from_inner(psbt: PartiallySignedTransactionInner) -> Result<Psbt<Version>, String> {
        match validate_psbt_inner(psbt) {
            Ok(()) => Ok(Psbt { inner: psbt }),
            Err(err) => Err(err),
        }
    }

    fn validate_psbt_inner(psbt: &PartiallySignedTransactionInner) -> Result<(), String> {
        match psbt.version {
            Version::PsbtV0 => {
                // Some code to validate Psbt as a version 0 Psbt
                // let valid = validate(psbt);
                if !valid {
                    Err(String::from("Error parsing psbtv0"))
                }
            }
            Version::Psbtv2 => {
                // Some code to validate Psbt as a version 2 Psbt
                // let valid = validate(psbt);
                if !valid {
                    Err(String::from("Error parsing psbtv2"))
                }
            }
        }
        Ok(())
    }

    pub fn add_input(&self, input: Input) -> Result<(), String> {
        // Validate the input according to the version
        if validate_input(input) {
            Ok(())
        } else {
            Err("Error validating input!")
        }
    }

    pub fn add_output(&self, output: Output) -> Result<(), String> {
        // Validate the output according to the version
        if validate_output(output) {
            Ok(())
        } else {
            Err("Error validating output!")
        }
    }

    fn validate_input(&self, input: &Input) -> bool {
        // Code to validate input based on the psbt version
        true
    }

    fn validate_output(&self, output: &Output) -> bool {
        // Code to validate output based on the psbt version
        true
    }

    pub fn to_inner(self) -> PartiallySignedTransactionInner {
        self.inner
    }
}
