Hello, I am Subhradeep Chakraborty, a Summer Of Bitcoin mentee working on the implementation of psbtv2 in rust-bitcoin as well as the rust-miniscript and mentored by @sanket1729. 

## Implementation of Psbtv2 & Breaking Changes

### Context
- [Tracking PSBT refactoring & PSBTv2 Epic](https://github.com/rust-bitcoin/rust-bitcoin/issues/1115)
- [BIP 174](https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki)
- [BIP 370](https://github.com/bitcoin/bips/blob/master/bip-0370.mediawiki)

### PartiallySignedTransactionInner

The following approach replaces the `PartiallySignedTransaction` with `PartiallySignedTransactionInner` where all the version specific fields are made `Option`. It comes with the flexibility to create both the Psbtv0 and Psbtv2 and do conversion between them. This is a big breaking change and requires existing implementations to be changed so to support the new psbtv2 standard.

It assumes [this PR regarding the Psbt Version](https://github.com/rust-bitcoin/rust-bitcoin/pull/1218) to be marged and all the following changes are proposed on top of this PR.

```rust
// PartiallySignedTrasanction --> PartiallySignedTransactionInner
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "actual_serde"))]
pub struct PartiallySignedTransactionInner {
    /// The unsigned transaction, scriptSigs and witnesses for each input must be empty.
    pub unsigned_tx: Option<Transaction>,
    /// The version number of this PSBT. If omitted, the version number is V0.
    /// See https://github.com/rust-bitcoin/rust-bitcoin/pull/1218
    pub version: Version,

    // ...

    /// The corresponding key-value map for each input in the unsigned transaction.
    pub inputs: Vec<Input>, // New Input, see below
    /// The corresponding key-value map for each output in the unsigned transaction.
    pub outputs: Vec<Output>, // New Output, see below

    // More new Psbtv2 Optional fields go here
    /// 32-bit little endian signed integer representing the
    /// version number of the transaction being created
    pub tx_version: Option<i32>,
    /// 32-bit little endian unsigned integer representing the transaction locktime
    /// to use if no inputs specify a required locktime.
    pub fallback_locktime: Option<u32>,
    /// 8 bit unsigned integer as a bitfield for various transaction modification flags
    pub tx_modifiable: Option<u8>, // or, Option<TxModifiable>
}
```

Introduction to a new `struct` named `Psbt` that internally stores and owns a `PartiallySignedTransactionInner` instance. A `Psbt` instance always gurantees that the underlying `inner` is validated.

```rust
pub struct Psbt {
    inner: PartiallySignedTransactionInner,
}
```

A `PartiallySignedTransactionInner` first needs to be created with all the fields (atleast the required ones) filled. The "Inner" instance can not be directly used anywhere, instead it must be validated first using the following method. The following factory method validates the psbt according to the `version` enum field of the inner and finally returns a new `Psbt` instance.

```rust
impl Psbt {
    pub fn from_inner(psbt: PartiallySignedTransactionInner) -> Result<Psbt, String> {
        match validate_psbt_inner(&psbt) {
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
```

### Inputs and Outputs

```rust
pub struct Input {
    // All existing Input fields
    // ...

    // Optional Psbtv2 fields
    // use crate::hash_type::Txid;
    pub previous_tx_id: Option<Txid>, // Required in PsbtV2.
    pub output_index: Option<u32>, // Required in PsbtV2
    // use crate::blockdata::transaction::Sequence
    pub sequence: Option<Sequence>, // Optional in PsbtV2, but not allowed in PsbtV0
    pub required_time_locktime: Option<u32>, // Optional in PsbtV2, not allowed in PsbtV0
    pub required_height_locktime: Option<u32>, // Optional in PsbtV2, not allowed in PsbtV0
}

/// A key-value map for an output of the corresponding index in the unsigned
/// transaction.
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "actual_serde"))]
pub struct Output {
    // All existing Output fields
    // ...

    // Psbtv2 compulsory fields
    pub amount: Option<Amount>, // use crate::Amount;
    // use crate::blockdata::script::ScriptBuf;
    pub script: Option<ScriptBuf>,
}
```

Inputs and Outputs are not directly verified. Instead while adding new inputs and outputs to the psbt we can validate them based on its version.

```rust
impl Psbt {
    pub fn add_input(&self, input: Input) -> Result<(), String> {
        // Validate the input according to this psbt version
        if input.validate(self.version) {
            Ok(())
        } else {
            Err("Error validating input!")
        }
    }

    pub fn add_output(&self, output: Output) -> Result<(), String> {
        // Validate the output according to this psbt version
        if output.validate(self.version) {
            Ok(())
        } else {
            Err("Error validating output!")
        }
    }
}
```

### Serialization & Deserialization

```rust
impl PartiallySignedTransactionInner {
    // Hide direct access to `PartiallySignedTransactionInner::serialize()`
    pub(crate) fn serialize(&self) -> Vec<u8> {
        // Code
    }
    
    // Hide direct access to `PartiallySignedTransactionInner::deserialize()`
    pub(crate) fn deserialize(&[u8]) -> Self {
        // The PsbtV0 parser needs to be updated a little
        // to recognize new PsbtV2 fields
    }
}

impl Psbt {
    /// Wrapper around the `ParitallySignedTransactionInner::serialize` function.
    ///
    /// Since only the validated Psbts can be allowed to be serialized and
    /// transmitted through the network, only the `Psbt::serialize()` funciton
    /// is to be used by the developers for the serialization. 
    pub fn serialize(&self) -> Vec<u8> {
        self.inner.serialize()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
        // Internally uses the `PartiallySignedTransactionInner::deserialize()`
        // function. The internal `deserialize` function decodes the bytes
        // without validation. In the next step, `Psbt::from_inner` does all
        // the validation and finally returns the decoded and validated Psbt
        let psbt_inner = PartiallySignedTransactionInner::deserialize(&bytes);
        Psbt::from_inner(psbt_inner)
    }
}
```

### Conversion between PsbtV0 and PsbtV2

It is possible to create a psbtv0 out of a valid psbtv2 and vice versa. We can create the unsigned transaction required in PsbtV0 using various fields available in PsbtV2. 

```rust
impl Psbt {
    pub fn get_v2(self) -> Self {
        match self.version {
            Version::PsbtV0 => {
                // Convert Psbt to Version 2 Psbt
                let v2_inputs = Vec::new<Input>();
                let v2_outputs = Vec::new<Output>();
                let unsigned_tx = self.unsigned_tx?;

                for input in self.inputs {
                    // All the PsbtV0 input fields
                    let v2_input = Input {
                        // Fill new PsbtV2 fields using
                        // `unsigned_tx.input`.
                        ..input // Rest of the fields
                    }

                    v2_inputs.push(v2_input);
                }

                for output in self.outputs {
                    let v2_output = Output {
                        // Fill up `amount` and `script` from
                        // `unsigned_tx.output`
                        ..output // Rest of the output fields
                    }

                    v2_outputs.push(v2_output);
                }

                let psbt_inner = PartiallySignedTransactionInner {
                    transaction: None,
                    verison: Version::PsbtV2,
                    inputs: v2_inputs,
                    outputs: v2_outputs,
                    // Other PsbtV2 fields
                    // tx_version,
                    // fallback_locktime,
                    // tx_modifiable,
                    ..self // Rest of the Psbt fields
                };

                Psbt::from_inner(psbt_inner)
            }
            Version::PsbtV2 => self
        }
    }

    pub fn get_v0(self) -> Self {
        match self.version {
            Version::PsbtV0 => self,
            Version::PsbtV2 => {
                // Convert PsbtV2 to Version 0 Psbt
                // Adding new inputs and outputs to PsbtV0 is not
                // allowed, hence, tx_version, fallback_locktime and
                // tx_modifiable flags will be discarded. All the
                // `Transaction` fields are constructable from PsbtV2.
                let v0_tx_inputs = Vec::new<TxIn>();
                let v0_tx_outputs = Vec::new<TxOut>();

                // Extract v0_inputs and v0_outputs from `self.inputs`
                // and `self.outputs`.

                let tx = Transaction {
                    version: self.tx_version,
                    lock_time: self.fallback_locktime,
                    input: v0_tx_inputs,
                    output: v0_tx_outputs,
                };

                let psbt_inner = PartiallySignedTransactionInner {
                    unsigned_tx: tx,
                    version: Version::PsbtV0,
                    inputs: v0_inputs, // After dropping PsbtV2 fields
                    outputs: v0_outputs, // After dropping PsbtV2 fields
                    ..self // Rest of the Psbt fields
                };

                Psbt::from_inner(psbt_inner)
            }
        }
    }
}
```
