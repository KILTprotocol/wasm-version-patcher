use clap::Parser;
use sc_executor::read_embedded_version;
use sc_executor_common::runtime_blob::RuntimeBlob;
use sp_version::{create_runtime_str, RuntimeVersion};
use std::fs::File;
use std::io::prelude::*;

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("kilt-spiritnet"),
    impl_name: create_runtime_str!("kilt-spiritnet"),
    authoring_version: 1,
    spec_version: 10730,
    impl_version: 0,
    apis: sp_version::create_apis_vec!([
        ([223u8, 106, 203, 104, 153, 7, 96, 155], 3u32),
        ([55u8, 227, 151, 252, 124, 145, 245, 228], 1u32),
        ([188u8, 157, 137, 144, 79, 91, 146, 63], 1u32),
        ([55u8, 200, 187, 19, 80, 169, 162, 168], 1u32),
        ([64u8, 254, 58, 212, 1, 248, 149, 154], 1u32),
        ([210u8, 188, 152, 151, 238, 208, 143, 21], 1u32),
        ([247u8, 139, 39, 139, 229, 63, 69, 76], 1u32),
        ([171u8, 60, 5, 114, 41, 31, 235, 139], 1u32),
        ([221u8, 113, 141, 92, 197, 50, 98, 212], 1u32),
        ([234u8, 147, 227, 241, 111, 61, 105, 98], 1u32)
    ]),
    transaction_version: 1,
    state_version: 0,
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input WASM
    #[arg(short, long)]
    input: String,

    /// Output WASM
    #[arg(short, long)]
    output: String,

    #[arg(short = 'x', long, default_value = "false")]
    hex: bool,
}

fn main() {
    let args = Args::parse();

    let mut file = File::open(args.input).expect("Failed to open input file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("Failed to read file");

    let wasm = sp_maybe_compressed_blob::decompress(
        &contents[..],
        sp_maybe_compressed_blob::CODE_BLOB_BOMB_LIMIT,
    )
    .expect("WASM should decompress");

    let old_blob = RuntimeBlob::new(&wasm).expect(
        "Embedded old blob is
	valid",
    );
    println!(
        "OLD spec version is {:#?}",
        read_embedded_version(&old_blob)
            .ok()
            .flatten()
            .expect("Reading embedded version works")
    );

    let fixed_wasm = sp_version::embed::embed_runtime_version(&wasm[..], VERSION)
        .expect("Failed to write version");

    let new_blob = RuntimeBlob::new(&fixed_wasm).expect(
        "Embedded old blob is
        valid",
    );
    println!(
        "NEW spec version is {:#?}",
        read_embedded_version(&new_blob)
            .ok()
            .flatten()
            .expect("Reading embedded version works")
    );

    let new_blob_comp = sp_maybe_compressed_blob::compress(
        &fixed_wasm[..],
        sp_maybe_compressed_blob::CODE_BLOB_BOMB_LIMIT,
    )
    .expect("WASM should decompress");

    if args.hex {
        let mut file = File::create(args.output).expect("Failed to open output file");
        file.write_all(b"0x").expect("Failed to write output file");
        file.write_all(hex::encode(&new_blob_comp[..]).as_bytes())
            .expect("Failed to write output file");
    } else {
        let mut file = File::create(args.output).expect("Failed to open output file");
        file.write_all(&new_blob_comp[..])
            .expect("Failed to write output file");
    }
}
