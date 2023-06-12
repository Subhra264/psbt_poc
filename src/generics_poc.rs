/// A Partially Signed Transaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "actual_serde"))]
pub struct PartiallySignedTransaction<V = PsbtUnchecked>
where
    V: PsbtValidation,
{
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
    // More Optional psbtv2 fields go here
}

pub trait PsbtValidation {
    const IS_VALIDATED: bool;
}

pub enum PsbtChecked {}
pub enum PsbtUnchecked {}

impl PsbtValidation for PsbtChecked {}
impl PsbtValidation for PsbtUnchecked {}

// pub struct Psbt<V = PsbtChecked>
// where
//     V: PsbtValidation,
// {
//     inner: Option<PartiallySignedTransactionInner>,
// }

impl<PsbtValidation> PartiallySignedTransaction<PsbtValidation> {
    pub fn validate(&self) -> Result<PartiallySignedTransaction<PsbtChecked>, String> {
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
}

impl PartiallySignedTransaction<PsbtChecked> {
    // Methods only available for the checked Psbt
    // ...
}
