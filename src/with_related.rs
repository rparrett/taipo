// TODO: Either replace usage with Bevy's upcoming Construct (or whatever) or use this
// from Bevy is https://github.com/bevyengine/bevy/pull/18961 is merged.

use bevy::{
    ecs::{relationship::Relationship, spawn::SpawnableList},
    prelude::*,
};

/// A [`SpawnableList`] that adds entities using an iterator of [`Entity`]:
///
/// ```
/// # use bevy_ecs::hierarchy::Children;
/// # use bevy_ecs::spawn::{Spawn, WithRelated, SpawnRelated};
/// # use bevy_ecs::name::Name;
/// # use bevy_ecs::world::World;
/// let mut world = World::new();
///
/// let child2 = world.spawn(Name::new("Child2")).id();
/// let child3 = world.spawn(Name::new("Child3")).id();
///
/// world.spawn((
///     Name::new("Root"),
///     Children::spawn((
///         Spawn(Name::new("Child1")),
///         WithRelated([child2, child3].into_iter()),
///     )),
/// ));
/// ```
pub struct WithRelated<I>(pub I);

impl<R: Relationship, I: Iterator<Item = Entity>> SpawnableList<R> for WithRelated<I> {
    fn spawn(self, world: &mut World, entity: Entity) {
        world
            .entity_mut(entity)
            .add_related::<R>(&self.0.collect::<Vec<_>>());
    }

    fn size_hint(&self) -> usize {
        self.0.size_hint().0
    }
}

/// A wrapper over an [`Entity`] indicating that an entity should be added.
/// This is intended to be used for hierarchical spawning via traits like [`SpawnableList`] and [`SpawnRelated`].
///
/// Also see the [`children`](crate::children) and [`related`](crate::related) macros that abstract over the [`Spawn`] API.
///
/// ```
/// # use bevy_ecs::hierarchy::Children;
/// # use bevy_ecs::spawn::{Spawn, WithOneRelated, SpawnRelated};
/// # use bevy_ecs::name::Name;
/// # use bevy_ecs::world::World;
/// let mut world = World::new();
///
/// let child1 = world.spawn(Name::new("Child1")).id();
///
/// world.spawn((
///     Name::new("Root"),
///     Children::spawn((
///         WithOneRelated(child1),
///     )),
/// ));
/// ```
pub struct WithOneRelated(pub Entity);

impl<R: Relationship> SpawnableList<R> for WithOneRelated {
    fn spawn(self, world: &mut World, entity: Entity) {
        world.entity_mut(entity).add_one_related::<R>(self.0);
    }

    fn size_hint(&self) -> usize {
        1
    }
}
