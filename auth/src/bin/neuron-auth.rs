// ---
// tags: neuron, auth, rust, cli
// crystal-type: source
// crystal-domain: comp
// ---
//! neuron-auth — keys, spells and claims of the neuron.
//!
//!   neuron-auth                       banner + help
//!   neuron-auth new                   generate a fresh wallet
//!   neuron-auth derive <spell…>    spell → addresses, pubkey, native neuron
//!   neuron-auth neuron <spell…>    native neuron id = Hemera(pubkey)
//!   neuron-auth claim  <spell…>    sign a migration claim (legacy key → native neuron)
//!   neuron-auth verify <claim>        verify a migration claim
//!
//! The bridge for migrating a Cosmos-SDK account (spacepussy first) into a
//! native neuron. `--bostrom` switches the address prefix from `pussy`.

use std::io::{self, IsTerminal};
use std::process::exit;

use neuron_auth::claim::{self, Claim};
use neuron_auth::{cosmos, spell};

// ── color ───────────────────────────────────────────────────────────────────

fn tty() -> bool {
    io::stdout().is_terminal()
}

fn paint(code: &str, s: &str) -> String {
    if tty() { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_string() }
}
fn dim(s: &str) -> String {
    paint("90", s)
}
fn cyan(s: &str) -> String {
    paint("36", s)
}
fn green(s: &str) -> String {
    paint("32", s)
}
fn yellow(s: &str) -> String {
    paint("33", s)
}
fn red(s: &str) -> String {
    paint("31", s)
}
fn bold(s: &str) -> String {
    paint("1", s)
}

const LOGO: &str = "\
\x1b[31m███╗   ███╗██╗   ██╗██████╗ ██████╗  █████╗ \x1b[0m
\x1b[33m████╗ ████║██║   ██║██╔══██╗██╔══██╗██╔══██╗\x1b[0m
\x1b[32m██╔████╔██║██║   ██║██║  ██║██████╔╝███████║\x1b[0m
\x1b[36m██║╚██╔╝██║██║   ██║██║  ██║██╔══██╗██╔══██║\x1b[0m
\x1b[34m██║ ╚═╝ ██║╚██████╔╝██████╔╝██║  ██║██║  ██║\x1b[0m
\x1b[35m╚═╝     ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝\x1b[0m";

fn banner() {
    if !tty() {
        return;
    }
    println!("{LOGO}");
    println!("{}", paint("37", "    the seal of the neuron"));
    println!(
        "{}",
        dim(
            "\n    secp256k1 · BIP-39/32 · coin type 118\n    \
             ripemd160 ∘ sha256 → bech32 · ADR-036 claims\n    \
             legacy key → Hemera(pubkey) · post-quantum ahead\n"
        )
    );
}

fn help() {
    println!(
        "{}\n  {}\n  {}\n  {}\n  {}\n  {}\n\n  {}\n    {}\n\n  {}",
        dim("commands"),
        format!("{}                    {}", bold("new"), dim("generate a fresh 24-word wallet")),
        format!("{}     {}", bold("derive <spell…>"), dim("spell → pussy/bostrom address, pubkey, neuron")),
        format!("{}     {}", bold("neuron <spell…>"), dim("native neuron id = Hemera(pubkey)")),
        format!("{}      {}", bold("claim <spell…>"), dim("sign a claim binding the legacy key → neuron")),
        format!("{}          {}", bold("verify <claim>"), dim("verify a migration claim")),
        dim("flags"),
        format!("{}   {}", bold("--bostrom"), dim("use the bostrom prefix instead of pussy")),
        dim("spell words follow the subcommand; default prefix is pussy"),
    );
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Print an error and exit non-zero.
fn die(msg: impl std::fmt::Display) -> ! {
    eprintln!("  {}: {msg}", red("error"));
    exit(1);
}

/// A labeled field line: dim label, bright value.
fn field(label: &str, value: &str) {
    println!("  {}  {}", dim(&format!("{label:<9}")), value);
}

// ── commands ────────────────────────────────────────────────────────────────

fn cmd_derive(spell: &str, hrp: &str) {
    let key = spell::cosmos_key(spell, "").unwrap_or_else(|e| die(e));
    let pubkey = cosmos::compressed(key.verifying_key());
    let addr = cosmos::address(&pubkey, hrp).unwrap_or_else(|e| die(e));
    let neuron = claim::neuron_of(&pubkey);
    field("account", &cyan(&addr));
    field("pubkey", &dim(&hex(&pubkey)));
    field("neuron", &green(&hex(&neuron)));
}

fn cmd_neuron(spell: &str) {
    let key = spell::cosmos_key(spell, "").unwrap_or_else(|e| die(e));
    let neuron = claim::neuron_of(&cosmos::compressed(key.verifying_key()));
    println!("  {}", green(&hex(&neuron)));
}

fn cmd_claim(spell: &str, hrp: &str) {
    let key = spell::cosmos_key(spell, "").unwrap_or_else(|e| die(e));
    let pubkey = cosmos::compressed(key.verifying_key());
    let neuron = claim::neuron_of(&pubkey);
    let c = claim::create(&key, hrp, neuron).unwrap_or_else(|e| die(e));
    field("account", &cyan(&c.address));
    field("neuron", &green(&hex(&c.neuron)));
    println!("  {} {}", dim("claim"), yellow(&c.encode()));
    println!("  {}", dim("↑ paste this into `neuron-auth verify`"));
}

fn cmd_verify(input: &str, hrp: &str) {
    let Some(c) = Claim::decode(input) else {
        die("malformed claim (expected: address pubkey neuron signature)");
    };
    if claim::verify(&c, hrp) {
        field("account", &cyan(&c.address));
        field("neuron", &green(&hex(&c.neuron)));
        println!("  {} {}", green("✓"), bold("claim verifies"));
    } else {
        println!("  {} {}", red("✗"), bold("claim does NOT verify"));
        exit(1);
    }
}

fn cmd_new(hrp: &str) {
    let spell = bip39::Mnemonic::generate(24).unwrap_or_else(|e| die(e));
    let phrase = spell.to_string();
    let key = spell::cosmos_key(&phrase, "").unwrap_or_else(|e| die(e));
    let pubkey = cosmos::compressed(key.verifying_key());
    let addr = cosmos::address(&pubkey, hrp).unwrap_or_else(|e| die(e));
    field("spell", &yellow(&phrase));
    field("account", &cyan(&addr));
    field("neuron", &green(&hex(&claim::neuron_of(&pubkey))));
    println!("  {}", dim("↑ write the spell down — it is the only key to this account"));
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    // --bostrom flag toggles the address prefix; strip it wherever it appears.
    let hrp = if let Some(i) = args.iter().position(|a| a == "--bostrom") {
        args.remove(i);
        cosmos::BOSTROM
    } else {
        cosmos::PUSSY
    };

    let cmd = args.first().cloned().unwrap_or_default();
    let rest = args.get(1..).unwrap_or(&[]).join(" ");

    match cmd.as_str() {
        "" | "help" | "-h" | "--help" => {
            banner();
            help();
        }
        "new" => cmd_new(hrp),
        "derive" | "address" => {
            if rest.is_empty() {
                die("usage: neuron-auth derive <spell…>");
            }
            cmd_derive(&rest, hrp);
        }
        "neuron" => {
            if rest.is_empty() {
                die("usage: neuron-auth neuron <spell…>");
            }
            cmd_neuron(&rest);
        }
        "claim" => {
            if rest.is_empty() {
                die("usage: neuron-auth claim <spell…>");
            }
            cmd_claim(&rest, hrp);
        }
        "verify" => {
            if rest.is_empty() {
                die("usage: neuron-auth verify <claim>");
            }
            cmd_verify(&rest, hrp);
        }
        other => die(format!("unknown command: {other}  (try: neuron-auth help)")),
    }
}
