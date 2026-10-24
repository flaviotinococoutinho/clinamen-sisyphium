//! Transition-only regimen selection (REQ-C-05).
use sisyphium::{
    consilium::{band_tick_ms, budget, consilium},
    hospes::{Facultates, Regimen},
    mundus::Mundus,
    verbum::Modus,
};

/// Exact reevaluation conditions of REQ-C-05; the seed is deliberately absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Clavis {
    facultates: Facultates,
    intervallum: u32,
    modus: Modus,
    pensum: u32,
    semantica: [bool; 6],
}
impl Clavis {
    /// Captures capabilities, budget, band and the six semantic predicates (REQ-C-05).
    fn new(facultates: &Facultates, world: &Mundus, energy: u8) -> Self {
        let present = world.verbum;
        let memory = world.memoria;
        Self {
            facultates: *facultates,
            intervallum: band_tick_ms(energy),
            modus: present.modus(),
            pensum: budget(facultates, energy),
            semantica: [
                memory.ramus() != 3,
                (13..=15).contains(&memory.eventus()),
                memory.clausurae() & 0x3ffe != 0,
                present.onus() > 128,
                present.tranquillitas() >= 48,
                present.conatus() > 200 && present.onus() > 200,
            ],
        }
    }
}

/// Volatile cached regimen, compared before and after evolution by the host (REQ-C-05).
#[derive(Debug, Default)]
pub struct Electio {
    electa: Option<(Clavis, Regimen)>,
    #[cfg(test)]
    quaestiones: usize,
}
impl Electio {
    /// Starts without a regimen, requiring exactly one initial search (REQ-C-05).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            electa: None,
            #[cfg(test)]
            quaestiones: 0,
        }
    }

    /// Searches only when a reevaluation condition changes (REQ-C-05).
    #[must_use]
    pub fn select(&mut self, facultates: &Facultates, world: &Mundus, energy: u8) -> Regimen {
        let key = Clavis::new(facultates, world, energy);
        if let Some((previous, regimen)) = self.electa {
            if previous == key {
                return regimen;
            }
        }
        let regimen = consilium(facultates, world, energy);
        self.electa = Some((key, regimen));
        #[cfg(test)]
        {
            self.quaestiones += 1;
        }
        regimen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every semantic transition performs a search, and a mere seed change does not (REQ-C-05).
    #[test]
    fn cache_transition_only() {
        let host = Facultates::default();
        let world = Mundus::genesis(1);
        let mut cache = Electio::new();
        let initial = cache.select(&host, &world, 255);
        assert_eq!(cache.select(&host, &world, 255), initial);
        let mut seed = world;
        seed.verbum = seed.verbum.with_semen(19);
        assert_eq!(cache.select(&host, &seed, 255), initial);
        assert_eq!(cache.quaestiones, 1);
        let _ = cache.select(&host, &world, 254);
        assert_eq!(cache.quaestiones, 2);
        let mut variants = [world; 7];
        variants[0].memoria = world.memoria.with_ramus(0);
        variants[1].memoria = world.memoria.with_eventus(13);
        variants[2].memoria = world.memoria.with_clausurae(2);
        variants[3].verbum = world.verbum.with_onus(129);
        variants[4].verbum = world.verbum.with_tranquillitas(48);
        // Keep the onus predicate already true to isolate the sixth semantic component.
        let mut excess = world;
        excess.verbum = world.verbum.with_onus(201).with_conatus(200);
        variants[5] = excess;
        variants[5].verbum = excess.verbum.with_conatus(201);
        variants[6].verbum = world.verbum.with_modus(Modus::Quies);
        for (index, variant) in variants.into_iter().enumerate() {
            let base = if index == 5 { excess } else { world };
            let _ = cache.select(&host, &base, 255);
            let count = cache.quaestiones;
            let _ = cache.select(&host, &variant, 255);
            assert_eq!(cache.quaestiones, count + 1, "transition {index}");
        }
        let count = cache.quaestiones;
        let _ = cache.select(&Facultates { width: 640, ..host }, &variants[6], 255);
        assert_eq!(cache.quaestiones, count + 1);
    }
}
