/// Optional strong typing for Bevy entities.

use bevy_ecs::{component::Component, world::{Mut, World}};
use moonshine_kind::{Instance, InstanceRef};

pub trait SpawnInstanceWorld {
    fn spawn_instance<T: Component>(&mut self, component: T) -> Mut<T>;

    fn spawn_instance_id<T: Component>(&mut self, component: T) -> Instance<T>;

    fn spawn_instance_ref<T: Component>(&mut self, component: T) -> InstanceRef<T>;
}

pub trait ComponentInstance {
    fn instance<T: Component>(&self, instance: Instance<T>) -> Option<&T>;

    fn instance_mut<T: Component>(&mut self, instance: Instance<T>) -> Option<Mut<T>>;
}

impl SpawnInstanceWorld for World {
    fn spawn_instance<T: Component>(&mut self, component: T) -> Mut<T> {
        let entity = self.spawn(component).id();
        self.get_mut(entity).unwrap()
    }

    fn spawn_instance_id<T: Component>(&mut self, component: T) -> Instance<T> {
        let entity = self.spawn(component);
        Instance::from_entity(entity.into()).unwrap()
    }

    fn spawn_instance_ref<T: Component>(&mut self, component: T) -> InstanceRef<T> {
        let entity = self.spawn(component).into();
        InstanceRef::from_entity(entity).unwrap()
    }
}

impl ComponentInstance for World {
    fn instance<T: Component>(&self, instance: Instance<T>) -> Option<&T> {
        self.get::<T>(instance.entity())
    }

    fn instance_mut<T: Component>(&mut self, instance: Instance<T>) -> Option<Mut<T>> {
        self.get_mut::<T>(instance.entity())
    }
}