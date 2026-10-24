//! Regimen cache equivalence on invalidating transitions (REQ-C-05).
use sisyphium::{consilium::consilium, hospes::Facultates, mundus::Mundus, verbum::Modus};
use sisyphium_hospes::electio::Electio;

/// Exercises the named host law against independent inputs (REQ-C-05, REQ-H-06, REQ-D-02).
#[test]
fn cached_transition_matches_core_search() {
    let mut cache = Electio::new();
    let mut host = Facultates::default();
    let mut world = Mundus::genesis(17);
    for energy in [255, 254, 179, 178, 77, 76, 39, 38, 0] {
        assert_eq!(
            cache.select(&host, &world, energy),
            consilium(&host, &world, energy)
        );
    }
    for mode in [
        Modus::Vigilia,
        Modus::Somnium,
        Modus::Quies,
        Modus::AmorFati,
    ] {
        world.verbum = world.verbum.with_modus(mode);
        assert_eq!(
            cache.select(&host, &world, 255),
            consilium(&host, &world, 255)
        );
    }
    host.cells = true;
    assert_eq!(
        cache.select(&host, &world, 255),
        consilium(&host, &world, 255)
    );
}
