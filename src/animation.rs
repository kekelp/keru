use crate::*;

pub(crate) const DT: f32 = 1.0 / 60.0;

const TUNING_RESCALE: f32 = 165.0 * 1.0 / 60.0;

const BASE_EXPONENTIAL_RATE: f32 = 5.0 * TUNING_RESCALE;

const SNAP_DISTANCE: f32 = 0.003;
const MIN_SPEED: f32 = 0.005 * TUNING_RESCALE;

const CONST_RATE_EXPONENT: f32 = 0.5;

const MIN_DT: f32 = 1.0 / 1000.0;

impl System {
    fn animation_dt(&self) -> f32 {
        self.frame_dt
    }

    pub(crate) fn update_frame_time(&mut self) {
        let now = T0.elapsed().as_secs_f32();
        let raw = now - self.t;
        let max_dt = self.monitor_frame_time * 2.0;
        self.frame_dt = if raw > max_dt { self.monitor_frame_time } else { raw.max(MIN_DT) };
        self.t = now;
    }

    pub(crate) fn anim_exp_speed(&self, speed: f32) -> f32 {
        (BASE_EXPONENTIAL_RATE * self.global_animation_speed * speed * self.animation_dt()).clamp(0.0, 1.0)
    }

    pub(crate) fn exp_tail_step_dist(&self, dist: f32, speed: f32, snap: f32, min: f32) -> (f32, bool) {
        let g = self.global_animation_speed;
        let effective = g * speed;
        if effective <= 0.0 || dist <= snap * effective {
            (dist, true)
        } else {
            let const_floor = min * speed * g.powf(CONST_RATE_EXPONENT) * (self.animation_dt() / DT);
            ((dist * self.anim_exp_speed(speed)).max(const_floor).min(dist), false)
        }
    }

    pub(crate) fn exp_tail_step(&self, current: f32, target: f32, speed: f32) -> (f32, bool) {
        let diff = target - current;
        let (step, done) = self.exp_tail_step_dist(diff.abs(), speed, SNAP_DISTANCE, MIN_SPEED);
        (current + step * diff.signum(), done)
    }

    pub(crate) fn pure_exp_step(&self, current: f32, target: f32, speed: f32, snap: f32) -> (f32, bool) {
        if self.global_animation_speed * speed <= 0.0 {
            return (target, true);
        }
        let diff = target - current;
        if diff.abs() <= snap {
            (target, true)
        } else {
            (current + diff * self.anim_exp_speed(speed), false)
        }
    }
}
