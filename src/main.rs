use clap::Parser;
use sc_executor::read_embedded_version;
use sc_executor_common::runtime_blob::RuntimeBlob;
use sp_version::RuntimeVersion;
use std::fs::File;
use std::io::prelude::*;

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

    /// Path to version file
    #[arg(short = 'r', long)]
    replacement: String,

    /// Write as hex
    #[arg(short = 'x', long, default_value = "false")]
    hex: bool,
}

fn main() {
    let args = Args::parse();

    let mut wasm_in_file = File::open(args.input).expect("Failed to open input file");
    let mut wasm_input = Vec::new();
    wasm_in_file
        .read_to_end(&mut wasm_input)
        .expect("Failed to read file");

    let mut version_in_file = File::open(args.replacement).expect("Failed to open input file");
    let mut version_input = String::new();
    version_in_file
        .read_to_string(&mut version_input)
        .expect("Failed to read file");

    // try decode hex if it starts with 0x
    if &wasm_input[..2] == b"0x" {
        wasm_input = hex::decode(&wasm_input[2..]).unwrap();
    }

    // try decompress
    let wasm = sp_maybe_compressed_blob::decompress(
        &wasm_input[..],
        sp_maybe_compressed_blob::CODE_BLOB_BOMB_LIMIT,
    )
    .expect("should be valid wasm");

    let old_blob = RuntimeBlob::new(&wasm).expect("Embedded old blob is invalid");
    println!(
        "OLD spec version is {}",
        serde_json::to_string(
            &read_embedded_version(&old_blob)
                .ok()
                .flatten()
                .expect("Reading embedded version should work")
        )
        .expect("should serialize")
    );

    let version: RuntimeVersion =
        ron::from_str(&version_input).expect("RuntimeVersion should be valid");

    let fixed_wasm = sp_version::embed::embed_runtime_version(&wasm[..], version)
        .expect("Failed to write version");

    let new_blob = RuntimeBlob::new(&fixed_wasm).expect("Embedded old blob is invalid");
    println!(
        "NEW spec version is {:?}",
        serde_json::to_string(
            &read_embedded_version(&new_blob)
                .ok()
                .flatten()
                .expect("Reading new embedded version failed")
        )
        .unwrap()
    );

    let new_blob_comp = sp_maybe_compressed_blob::compress(
        &new_blob.serialize()[..],
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
