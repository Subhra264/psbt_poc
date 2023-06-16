use input::Input;
use output::Output;
use poc::{PartiallySignedTransactionInner, Psbt};

fn main() {
    let psbt_inner = PartiallySignedTransactionInner {
        // initialization of fields
    };

    let new_input = Input {};

    let new_output = Output {};

    let psbt = Psbt::from_inner(psbt_inner);
}
