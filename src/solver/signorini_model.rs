use std::ops::Range;
use na::{self, DVector, Real, Unit};

use ncollide::query::TrackedContact;
use detection::BodyContactManifold;
use solver::helper;
use solver::{ConstraintSet, ContactModel, ForceDirection, ImpulseCache, IntegrationParameters,
             UnilateralConstraint, UnilateralGroundConstraint};
use object::{BodyHandle, BodySet};
use math::{Point, Vector};

pub struct SignoriniModel<N: Real> {
    impulses: ImpulseCache<N>,
    vel_ground_rng: Range<usize>,
    vel_rng: Range<usize>,
}

impl<N: Real> SignoriniModel<N> {
    pub fn new() -> Self {
        SignoriniModel {
            impulses: ImpulseCache::new(),
            vel_ground_rng: 0..0,
            vel_rng: 0..0,
        }
    }

    pub fn build_constraint(
        params: &IntegrationParameters<N>,
        bodies: &BodySet<N>,
        ext_vels: &DVector<N>,
        b1: BodyHandle,
        b2: BodyHandle,
        c: &TrackedContact<Point<N>>,
        deepest_normal: &Unit<Vector<N>>,
        margin1: N,
        margin2: N,
        impulse: N,
        cache_id: usize,
        ground_jacobian_id: &mut usize,
        jacobian_id: &mut usize,
        jacobians: &mut [N],
        constraints: &mut ConstraintSet<N>,
    ) -> bool {
        let b1 = bodies.body_part(b1);
        let b2 = bodies.body_part(b2);

        let assembly_id1 = b1.parent_companion_id();
        let assembly_id2 = b2.parent_companion_id();

        let mut depth = c.contact.depth + margin1 + margin2;
        let normal = if true {
            // if depth >= N::zero() {
            depth *= na::dot(deepest_normal.as_ref(), c.contact.normal.as_ref());
            *deepest_normal
        } else {
            c.contact.normal
        };

        let mut geom = helper::constraint_pair_geometry(
            &b1,
            &b2,
            assembly_id1,
            assembly_id2,
            &(c.contact.world1 + normal.unwrap() * margin1),
            &(c.contact.world2 - normal.unwrap() * margin2),
            &ForceDirection::Linear(normal),
            ext_vels,
            ground_jacobian_id,
            jacobian_id,
            jacobians,
        );

        if depth < N::zero() {
            geom.rhs += -depth / params.dt;
        } else {
            let restitution = N::zero(); // FIXME: (rb1.restitution() + rb2.restitution()) * na::convert(0.5) * dvel;
            let stabilization = -depth * params.erp / params.dt;

            geom.rhs += na::inf(&restitution, &stabilization);
        }

        let warmstart = impulse * params.warmstart_coeff;

        if geom.is_ground_constraint() {
            constraints
                .velocity
                .unilateral_ground
                .push(UnilateralGroundConstraint::new(geom, warmstart, cache_id));

            true
        } else {
            constraints
                .velocity
                .unilateral
                .push(UnilateralConstraint::new(geom, warmstart, cache_id));

            false
        }
    }
}

impl<N: Real> ContactModel<N> for SignoriniModel<N> {
    fn nconstraints(&self, c: &BodyContactManifold<N>) -> usize {
        c.manifold.len()
    }

    fn build_constraints(
        &mut self,
        params: &IntegrationParameters<N>,
        bodies: &BodySet<N>,
        ext_vels: &DVector<N>,
        manifolds: &[BodyContactManifold<N>],
        ground_jacobian_id: &mut usize,
        jacobian_id: &mut usize,
        jacobians: &mut [N],
        constraints: &mut ConstraintSet<N>,
    ) {
        let id_vel_ground = constraints.velocity.unilateral_ground.len();
        let id_vel = constraints.velocity.unilateral.len();

        for manifold in manifolds {
            let deepest_contact_normal = &manifold.deepest_contact().contact.normal;
            for c in manifold.contacts() {
                let _ = Self::build_constraint(
                    params,
                    bodies,
                    ext_vels,
                    manifold.b1,
                    manifold.b2,
                    c,
                    deepest_contact_normal,
                    manifold.margin1,
                    manifold.margin2,
                    self.impulses.get(c.id),
                    self.impulses.entry_id(c.id),
                    ground_jacobian_id,
                    jacobian_id,
                    jacobians,
                    constraints,
                );
            }
        }

        self.vel_ground_rng = id_vel_ground..constraints.velocity.unilateral_ground.len();
        self.vel_rng = id_vel..constraints.velocity.unilateral.len();
    }

    fn cache_impulses(&mut self, constraints: &ConstraintSet<N>) {
        let ground_contacts = &constraints.velocity.unilateral_ground[self.vel_ground_rng.clone()];
        let contacts = &constraints.velocity.unilateral[self.vel_rng.clone()];

        for c in ground_contacts {
            self.impulses[c.cache_id] = c.impulse;
        }

        for c in contacts {
            self.impulses[c.cache_id] = c.impulse;
        }
    }
}
