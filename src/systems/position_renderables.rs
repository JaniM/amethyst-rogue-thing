use crate::components::{AnimateMovement, WorldPosition};
use amethyst::{
    core::{timing::Time, transform::Transform},
    ecs::{prelude::*, SystemData as _},
};

#[derive(Default)]
pub struct PositionRenderablesSystem {
    reader: Option<ReaderId<ComponentEvent>>,
}

#[derive(SystemData)]
pub struct SystemData<'s> {
    worldpos: ReadStorage<'s, WorldPosition>,
    transform: WriteStorage<'s, Transform>,
    animate: WriteStorage<'s, AnimateMovement>,
    time: Read<'s, Time>,
}

impl<'s> System<'s> for PositionRenderablesSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut dirty = BitSet::new();

        for event in data.worldpos.channel().read(self.reader.as_mut().unwrap()) {
            match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    dirty.add(*id);
                }
                _ => {}
            }
        }

        for (wp, transform, id, ()) in
            (&data.worldpos, &mut data.transform, &dirty, !&data.animate).join()
        {
            let nx = (wp.x as f32) * 50.0;
            let ny = (wp.y as f32) * 50.0;
            transform.set_x(nx);
            transform.set_y(ny);
            println!(
                "Position {} from world {}, {} to screen {}, {}",
                id, wp.x, wp.y, nx, ny
            );
        }

        for (wp, _, anim) in (&data.worldpos, &dirty, &mut data.animate).join() {
            anim.animate(anim.to.unwrap_or(*wp), *wp);
        }

        for (transform, anim) in (&mut data.transform, &mut data.animate).join() {
            let (from, to) = match (anim.from, anim.to) {
                (Some(from), Some(to)) => (from, to),
                _ => continue,
            };
            let d = (anim.used_time / anim.time).min(1.0);
            let nx = (to.x as f32 * d + from.x as f32 * (1.0 - d)) * 50.0;
            let ny = (to.y as f32 * d + from.y as f32 * (1.0 - d)) * 50.0;
            transform.set_x(nx);
            transform.set_y(ny);
            println!("Position to screen {}, {}", nx, ny);
            anim.used_time += data.time.delta_seconds();
            if d >= 1.0 {
                anim.reset();
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(WriteStorage::<WorldPosition>::fetch(&res).register_reader());
    }
}
