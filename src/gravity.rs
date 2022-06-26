use crate::distance::{Distance, Velocity};
use crate::time::{Tick, Time};
use crate::{coders::*, graph::*, property::*};
use num::Roots;

/// Gravitational acceleration given by 1 mass on 1 distance.
const GRAVITATIONAL_CONSTANT: f64 = unimplemented!();

type MassTy = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Mass(pub MassTy);

// TODO: make macro for this

impl Property for Mass {
    const ID: PropertyId = PropertyId::new(1);
}

impl Encode for Mass {
    const MAX_LEN: usize = <MassTy as Encode>::MAX_LEN;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for Mass {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}

type GravityAttractionTy = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct GravityAttraction(pub GravityAttractionTy);

impl GravityAttraction {
    pub fn none_if_zero(self) -> Option<Self> {
        if self.0 == 0 {
            None
        } else {
            Some(self)
        }
    }
}

// TODO: make macro for this

impl Property for GravityAttraction {
    const ID: PropertyId = PropertyId::new(2);
}

impl Encode for GravityAttraction {
    const MAX_LEN: usize = <GravityAttractionTy as Encode>::MAX_LEN;

    fn encode(&self, buffer: &mut [u8; Self::MAX_LEN]) -> usize {
        unimplemented!()
    }
}

impl Decode for GravityAttraction {
    fn decode(buffer: &[u8]) -> Self {
        unimplemented!()
    }
}

// TODO: Build pattern/pipleine for this. Omogučilo bi mapiranje ovisnosti između logika, te
// TODO: optimiziranije izvođenje pomoću grupiranja operacija i njihovo obavljanje u jednom prolazu,
// TODO: filtracija, etc... Rate of update, ...
pub fn compute_attraction<'a>(vertice: &impl Vertice<'a>) {
    if let Some(m) = Mass::of(vertice) {
        for mut edge in vertice.edges() {
            if let Some(d) = Distance::of(&edge) {
                // Function
                let ga = GravityAttraction(
                    (m.0 as u128 / (d.0 as u128).pow(2)).min(u64::MAX as u128) as u64,
                );

                //? Note: držanje efekt podatka s uzrok podacima ima smisla:
                //?       * Efekti su onda lokalni s uzrocima što znaci da nestankom uzroka efekt također sam nestaje.
                //?       * Daje mrežu urok -> efekt koja se može pratiti da se generira/loada potrebni vertice.
                //?       * Izmjene podataka ce biti većinom lokalne, samo mjenjajući lokalne cached podatke. (vertice, [-edge->])
                edge.update(ga.none_if_zero());
            }
        }
    }
}

pub trait Context: Container {}

//TODO: Maybe context and vertice could be joined in a single abstraction/vertice object.
pub fn update_velocity<'a>(vertice: &impl Vertice<'a>, context: &impl Context) {
    if let Some(tick) = Tick::of(vertice).or_else(|| Tick::of(context)) {
        for mut edge in vertice.edges() {
            if let Some(v) = Velocity::of(&edge) {
                if let Some(ga) = GravityAttraction::of(&edge.opposite()) {
                    // Function
                    let a = -((GRAVITATIONAL_CONSTANT * tick.0 as f64) * ga.0 as f64) as i64;
                    let v = Velocity(v.0.saturating_add(a));

                    edge.update(v.none_if_zero());
                }
            }
        }
    }
}

pub fn update_distance<'a>(vertice: &impl Vertice<'a>, context: &impl Context) {
    if let Some(tick) = Tick::of(vertice).or_else(|| Tick::of(context)) {
        for mut edge in vertice.edges() {
            if let Some(v) = Velocity::of(&edge) {
                if let Some(d) = Distance::of(&edge) {
                    // Function
                    let a = d.0 as i128;
                    let a_new = a
                        .saturating_add(v.0 as i128 * tick.0 as i128)
                        .min(u64::MAX as i128)
                        .max(0);
                    let delta_a = a_new - a;

                    //? Note: Neka za sada postoji samo jedna brzina u verticu.
                    //? Note: Neka za sada nije došlo do sudara ni prebacivanja udaljenosti.
                    // unimplemented!()

                    for other in vertice.edges().filter(|other| other != edge) {
                        if let Some(c) = Distance::of(&other) {
                            if let Some(b) = distance(edge.to(), other.to()) {
                                // c_new = (delta_a * (a_new + a - a^2 - b^2 + c^2) + c^2).sqrt()

                                // Pow 2
                                let a_pow2 = (a as i128).pow(2);
                                let b_pow2 = (b.0 as i128).pow(2);
                                let c_pow2 = (c.0 as i128).pow(2);

                                // a_new + a - a^2 - b^2 + c^2
                                let sum = (c_pow2 - a_pow2).saturating_add(a + a_new) - b_pow2;

                                // delta_a * sum + c^2
                                let inner = delta_a.saturating_mul(sum).saturating_add(c_pow2);

                                // c_new
                                let c_new = if inner >= 0 {
                                    inner.sqrt() as u64
                                } else {
                                    // Collision or overshot
                                    0
                                };

                                other.update(Distance(c_new));
                                other.opposite().update(Distance(c_new));
                            }
                        }
                    }

                    edge.update(Distance(a_new as u64));
                    edge.opposite().update(Distance(a_new as u64));
                }
            }
        }
    }
}

/// Determines distance between the two vertices, if it exists.
pub fn distance(a: &impl Vertice, b: &impl Vertice) -> Option<Distance> {
    unimplemented!()
}
