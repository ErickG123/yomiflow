/// Pure business logic for Free Spaced Repetition Scheduler (FSRS v4/v5 inspired)
/// State: 0 = New, 1 = Learning/Relearning, 2 = Review
/// Rating: 1 = Again, 2 = Hard, 3 = Good, 4 = Easy

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FsrsParameters {
    pub w: [f64; 17],
}

impl Default for FsrsParameters {
    fn default() -> Self {
        // Standard FSRS-4.5 default weight matrix
        Self {
            w: [
                0.40255, 1.18385, 3.173,
                15.69105, // Initial stabilities for Again, Hard, Good, Easy
                7.1949, 0.5345, // Initial difficulties
                1.4604, 0.0046, // Difficulty update
                1.5457, 0.1192, 1.0192, // Stability update on success
                1.9395, 0.11, 0.296, 0.2269, // Stability update on failure
                0.2315, 2.9898, // Retrievability & Interval
            ],
        }
    }
}

pub struct FsrsCardState {
    pub stability: f64,
    pub difficulty: f64,
    pub reps: i32,
    pub state: i32,
    pub due_date: i64,
}

pub fn calculate_fsrs_step(
    rating: i32,
    current_stability: f64,
    current_difficulty: f64,
    current_reps: i32,
    current_state: i32,
    last_review: i64,
    now: i64,
) -> Result<FsrsCardState, String> {
    if !(1..=4).contains(&rating) {
        return Err("Rating must be between 1 (Again) and 4 (Easy)".to_string());
    }

    let params = FsrsParameters::default();

    // 1. Initial review (New Card, reps == 0 or state == 0)
    if current_reps == 0 || current_state == 0 {
        let init_s = params.w[(rating - 1) as usize];
        let init_d = (params.w[4] - (rating as f64 - 3.0) * params.w[5]).clamp(1.0, 10.0);

        let (interval_secs, next_state) = match rating {
            1 => (10 * 60, 1),                         // Again: 10 minutes (Learning)
            2 => (12 * 3600, 1),                       // Hard: 12 hours (Learning)
            3 => ((init_s * 86400.0) as i64, 2),       // Good: based on stability in days
            4 => ((init_s * 1.5 * 86400.0) as i64, 2), // Easy: enhanced stability
            _ => unreachable!(),
        };

        return Ok(FsrsCardState {
            stability: init_s,
            difficulty: init_d,
            reps: 1,
            state: next_state,
            due_date: now + interval_secs.max(600),
        });
    }

    // 2. Subsequent reviews: Calculate elapsed days since last review
    let elapsed_days = ((now.saturating_sub(last_review)) as f64 / 86400.0).max(0.1);

    // Retrievability (forgetting curve): R = (1 + FACTOR * (t / S))^-DECAY
    let retrievability =
        (1.0 + 19.0 * elapsed_days / (9.0 * current_stability.max(0.1))).powf(-0.5);

    // Update Difficulty (mean reversion toward w[4])
    let mean_reversion = 0.05 * (params.w[4] - current_difficulty);
    let delta_d = -params.w[6] * (rating as f64 - 3.0);
    let new_difficulty = (current_difficulty + delta_d + mean_reversion).clamp(1.0, 10.0);

    // Update Stability
    let (new_stability, next_state, interval_secs) = match rating {
        1 => {
            // Failure / Lapse
            let s_lapse = params.w[11]
                * current_difficulty.powf(-params.w[12])
                * ((current_stability + 1.0).powf(params.w[13]) - 1.0)
                * (-(1.0 - retrievability) * params.w[14]).exp();
            let s = s_lapse.max(0.1);
            (s, 1, 10 * 60) // 10 minutes for lapse learning
        }
        2 => {
            // Hard review
            let s_inc = (params.w[8] * (11.0 - new_difficulty) * current_stability.powf(-params.w[9])
                * ((1.0 - retrievability) * params.w[10]).exp()
                * 0.8) // Hard penalty
                + 1.0;
            let s = (current_stability * s_inc).max(0.5);
            let interval = (s * 86400.0 * 0.8) as i64;
            (s, 2, interval.max(12 * 3600))
        }
        3 => {
            // Good review
            let s_inc = (params.w[8]
                * (11.0 - new_difficulty)
                * current_stability.powf(-params.w[9])
                * ((1.0 - retrievability) * params.w[10]).exp())
                + 1.0;
            let s = (current_stability * s_inc).max(1.0);
            let interval = (s * 86400.0) as i64;
            (s, 2, interval.max(24 * 3600))
        }
        4 => {
            // Easy review
            let s_inc = (params.w[8] * (11.0 - new_difficulty) * current_stability.powf(-params.w[9])
                * ((1.0 - retrievability) * params.w[10]).exp()
                * 1.3) // Easy bonus
                + 1.0;
            let s = (current_stability * s_inc).max(2.0);
            let interval = (s * 86400.0 * 1.3) as i64;
            (s, 2, interval.max(3 * 86400))
        }
        _ => unreachable!(),
    };

    Ok(FsrsCardState {
        stability: new_stability,
        difficulty: new_difficulty,
        reps: current_reps + 1,
        state: next_state,
        due_date: now + interval_secs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsrs_initial_ratings() {
        let now = 1_000_000;
        let again = calculate_fsrs_step(1, 0.0, 0.0, 0, 0, now, now).unwrap();
        assert_eq!(again.state, 1);
        assert_eq!(again.due_date, now + 10 * 60);

        let good = calculate_fsrs_step(3, 0.0, 0.0, 0, 0, now, now).unwrap();
        assert_eq!(good.state, 2);
        assert!(good.due_date > now + 24 * 3600);
    }

    #[test]
    fn test_fsrs_progression_good() {
        let now = 1_000_000;
        let step1 = calculate_fsrs_step(3, 0.0, 0.0, 0, 0, now, now).unwrap();
        let step2 = calculate_fsrs_step(
            3,
            step1.stability,
            step1.difficulty,
            step1.reps,
            step1.state,
            now,
            now + 86400,
        )
        .unwrap();

        assert!(step2.stability > step1.stability);
        assert_eq!(step2.reps, 2);
    }
}
