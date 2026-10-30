// SPDX-License-Identifier: MIT OR Apache-2.0
//! `docs/KISS.md`: the KISS-ANNOUNCE 7.3-0003 worked vector, run through `evaluate`. The adapter
//! (here: `live_word` and `matrix_from`) is test code; CapAnt itself knows nothing about KISS.

use capant::*;

const FEAT: u64 = ((1u64 << 48) - 1) & !((1u64 << 24) - 1);
const CONSUMER: u64 = 0x0031_0000_0000_0001;
const PROVIDER: u64 = 0x0091_0003_0000_0003;
const LIVE: u64 = 0x0011_0003_0000_0001;

/// Known bits only; any other set bit gets no row (7.2-0007).
const KNOWN: [(u32, &str); 8] = [
    (0, "kiss.ext.dlpack_v1"),
    (1, "kiss.ext.dlpack_mx"),
    (32, "kiss.feat.provision_on_request"),
    (33, "kiss.feat.contract_query"),
    (48, "kiss.sub.speaks_announce"),
    (52, "kiss.sub.speaks_synth"),
    (53, "kiss.sub.speaks_consume"),
    (55, "kiss.sub.speaks_contract"),
];

fn live_word(local: u64, remote: u64, provider: u64) -> u64 {
    (local & remote & !FEAT) | (provider & FEAT)
}

fn id(s: &str) -> CapabilityId {
    CapabilityId::new(s).unwrap()
}

fn matrix_from(word: u64) -> Matrix {
    Matrix::from_rows(
        KNOWN
            .iter()
            .map(|&(bit, name)| {
                if word >> bit & 1 == 1 {
                    Row::capability(id(name), "peer", "1", State::Yes)
                        .evidence(Evidence::probe("announce:handshake"))
                } else {
                    Row::capability(id(name), "peer", "1", State::No(Reason::Missing))
                }
            })
            .collect(),
    )
}

fn unmet_for(word: u64, wanted: &[&str]) -> Vec<String> {
    let peer = Provider::new("peer", vec![Surface::new("peer", "1", Config::new())]).unwrap();
    let need = Need::new(wanted.iter().map(|s| id(s)).collect());
    let eval = evaluate(
        &[peer],
        &need,
        &matrix_from(word),
        &Context::new(0, Config::new()),
    )
    .unwrap();
    match eval.check("peer") {
        Ok(_) => Vec::new(),
        Err(e) => vec![e.to_string()],
    }
}

#[test]
fn worked_vector_live_word_and_plain_and() {
    assert_eq!(live_word(CONSUMER, PROVIDER, PROVIDER), LIVE);
    assert_eq!(
        live_word(PROVIDER, CONSUMER, PROVIDER),
        LIVE,
        "roles exchanged"
    );
    // Negative control: the plain AND loses both FEAT bits, so the rule is not it.
    assert_eq!(CONSUMER & PROVIDER, 0x0011_0000_0000_0001);
}

#[test]
fn engine_satisfies_the_live_feat_need_and_names_the_failed_one() {
    let wanted = [
        "kiss.feat.contract_query",
        "kiss.sub.speaks_synth",
        "kiss.ext.dlpack_v1",
    ];
    assert!(unmet_for(LIVE, &wanted).is_empty());
    // Negative control: the plain-AND word fails the same need, and the text names it.
    let why = unmet_for(CONSUMER & PROVIDER, &wanted).join("");
    assert!(why.contains("kiss.feat.contract_query [no"), "{why}");
    // Not live on either side: SUB 55 is the provider's only, EXT 1 is the provider's only.
    let why = unmet_for(LIVE, &["kiss.sub.speaks_contract"]).join("");
    assert!(why.contains("kiss.sub.speaks_contract"), "{why}");
}

#[test]
fn consumer_feat_bit_never_changes_the_answer() {
    let flipped = CONSUMER ^ (1 << 33);
    assert_eq!(live_word(flipped, PROVIDER, PROVIDER), LIVE);
    let wanted = ["kiss.feat.contract_query"];
    assert_eq!(
        unmet_for(live_word(flipped, PROVIDER, PROVIDER), &wanted),
        unmet_for(LIVE, &wanted)
    );
    // Negative control: a provider that clears the bit does change it.
    let cleared = PROVIDER & !(1 << 33);
    assert!(!unmet_for(live_word(CONSUMER, cleared, cleared), &wanted).is_empty());
}

#[test]
fn unknown_bit_costs_nothing() {
    let wanted = ["kiss.feat.contract_query"];
    assert_eq!(
        unmet_for(LIVE | 1 << 60 | 1 << 30, &wanted),
        unmet_for(LIVE, &wanted)
    );
}
